use crate::util::*;
use crate::symbols::*;
use crate::tree::{calculate_bit_lengths};
use crate::lz77::ZopfliLZ77Store;

/// Bit writer for constructing DEFLATE streams
pub struct BitWriter {
    data: Vec<u8>,
    bit_pointer: u8,  // 0-7, current bit position in last byte
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter {
            data: Vec::new(),
            bit_pointer: 0,
        }
    }

    /// Add a single bit to the output stream
    pub fn add_bit(&mut self, bit: u8) {
        if self.bit_pointer == 0 {
            self.data.push(0);
        }
        let byte_index = self.data.len() - 1;
        self.data[byte_index] |= (bit & 1) << self.bit_pointer;
        self.bit_pointer = (self.bit_pointer + 1) & 7;
    }

    /// Add multiple bits to the output stream
    pub fn add_bits(&mut self, symbol: u32, length: u32) {
        for i in 0..length {
            let bit = ((symbol >> i) & 1) as u8;
            self.add_bit(bit);
        }
    }

    /// Add Huffman bits (with inverted bit order as per DEFLATE spec)
    pub fn add_huffman_bits(&mut self, symbol: u32, length: u32) {
        for i in 0..length {
            let bit = ((symbol >> (length - i - 1)) & 1) as u8;
            self.add_bit(bit);
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_bit_pointer(&self) -> u8 {
        self.bit_pointer
    }
}

/// Gets the fixed Huffman tree as per DEFLATE spec
/// Returns lengths for literal/length codes and distance codes
fn get_fixed_tree() -> ([u32; ZOPFLI_NUM_LL], [u32; ZOPFLI_NUM_D]) {
    let mut ll_lengths = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths = [0u32; ZOPFLI_NUM_D];
    
    // Fixed literal/length tree lengths as per DEFLATE spec
    for i in 0..144 { ll_lengths[i] = 8; }
    for i in 144..256 { ll_lengths[i] = 9; }
    for i in 256..280 { ll_lengths[i] = 7; }
    for i in 280..288 { ll_lengths[i] = 8; }
    
    // Fixed distance tree lengths
    for i in 0..32 { d_lengths[i] = 5; }
    
    (ll_lengths, d_lengths)
}

/// Ensures there are at least 2 distance codes to support buggy decoders.
fn patch_distance_codes_for_buggy_decoders(d_lengths: &mut [u32; ZOPFLI_NUM_D]) {
    let mut num_dist_codes = 0;
    
    // Count non-zero distance codes (ignore the two unused codes from spec)
    for i in 0..30 {
        if d_lengths[i] != 0 {
            num_dist_codes += 1;
        }
        if num_dist_codes >= 2 {
            return; // Two or more codes is fine
        }
    }
    
    if num_dist_codes == 0 {
        d_lengths[0] = 1;
        d_lengths[1] = 1;
    } else if num_dist_codes == 1 {
        d_lengths[if d_lengths[0] != 0 { 1 } else { 0 }] = 1;
    }
}

/// Calculate block symbol size for small blocks (iterate through each symbol)
fn calculate_block_symbol_size_small(
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize
) -> usize {
    let mut result = 0;
    
    for i in lstart..lend {
        assert!(i < lz77.size());
        assert!(lz77.litlens()[i] < 259);
        
        if lz77.dists()[i] == 0 {
            // Literal
            result += ll_lengths[lz77.litlens()[i] as usize] as usize;
        } else {
            // Length-distance pair
            let ll_symbol = get_length_symbol(lz77.litlens()[i] as i32) as usize;
            let d_symbol = get_dist_symbol(lz77.dists()[i] as i32) as usize;
            result += ll_lengths[ll_symbol] as usize;
            result += d_lengths[d_symbol] as usize;
            result += get_length_symbol_extra_bits(ll_symbol as i32) as usize;
            result += get_dist_symbol_extra_bits(d_symbol as i32) as usize;
        }
    }
    
    result += ll_lengths[256] as usize; // End symbol
    result
}

/// Calculate block symbol size using precomputed histograms
fn calculate_block_symbol_size_given_counts(
    ll_counts: &[usize; ZOPFLI_NUM_LL],
    d_counts: &[usize; ZOPFLI_NUM_D],
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize
) -> usize {
    // For very small blocks, fall back to iteration
    if lstart + ZOPFLI_NUM_LL * 3 > lend {
        return calculate_block_symbol_size_small(ll_lengths, d_lengths, lz77, lstart, lend);
    }
    
    let mut result = 0;
    
    // Literal symbols (0-255)
    for i in 0..256 {
        result += ll_lengths[i] as usize * ll_counts[i];
    }
    
    // Length symbols (257-285)
    for i in 257..286 {
        result += ll_lengths[i] as usize * ll_counts[i];
        result += get_length_symbol_extra_bits(i as i32) as usize * ll_counts[i];
    }
    
    // Distance symbols (0-29)
    for i in 0..30 {
        result += d_lengths[i] as usize * d_counts[i];
        result += get_dist_symbol_extra_bits(i as i32) as usize * d_counts[i];
    }
    
    result += ll_lengths[256] as usize; // End symbol
    result
}

/// Calculate block symbol size, choosing between small and histogram-based calculation
fn calculate_block_symbol_size(
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize
) -> usize {
    if lstart + ZOPFLI_NUM_LL * 3 > lend {
        calculate_block_symbol_size_small(ll_lengths, d_lengths, lz77, lstart, lend)
    } else {
        let mut ll_counts = [0usize; ZOPFLI_NUM_LL];
        let mut d_counts = [0usize; ZOPFLI_NUM_D];
        lz77.get_histogram(lstart, lend, &mut ll_counts, &mut d_counts);
        calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, ll_lengths, d_lengths, lz77, lstart, lend)
    }
}

/// Get dynamic Huffman tree lengths for a block
fn get_dynamic_lengths(
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
) -> (f64, [u32; ZOPFLI_NUM_LL], [u32; ZOPFLI_NUM_D]) {
    let mut ll_counts = [0usize; ZOPFLI_NUM_LL];
    let mut d_counts = [0usize; ZOPFLI_NUM_D];
    
    lz77.get_histogram(lstart, lend, &mut ll_counts, &mut d_counts);
    ll_counts[256] = 1; // End symbol
    
    let mut ll_lengths = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths = [0u32; ZOPFLI_NUM_D];
    
    let _ = calculate_bit_lengths(&ll_counts, 15, &mut ll_lengths);
    let _ = calculate_bit_lengths(&d_counts, 15, &mut d_lengths);
    
    patch_distance_codes_for_buggy_decoders(&mut d_lengths);
    
    // For now, return a simplified tree size calculation
    // TODO: Implement full RLE optimization (TryOptimizeHuffmanForRle)
    let tree_size = calculate_tree_size_simple(&ll_lengths, &d_lengths);
    let symbol_size = calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, &ll_lengths, &d_lengths, lz77, lstart, lend);
    
    (tree_size + symbol_size as f64, ll_lengths, d_lengths)
}

/// Simplified tree size calculation (placeholder for full RLE optimization)
fn calculate_tree_size_simple(ll_lengths: &[u32; ZOPFLI_NUM_LL], d_lengths: &[u32; ZOPFLI_NUM_D]) -> f64 {
    // This is a simplified version that estimates tree encoding cost
    // The actual implementation uses RLE encoding to minimize the tree description
    
    // Count non-zero lengths
    let mut hlit = 29; // max HLIT value (ZOPFLI_NUM_LL - 257 - 1)
    let mut hdist = 29; // max distance codes - 1
    
    // Trim trailing zeros for literal/length codes
    while hlit > 0 && ll_lengths[257 + hlit - 1] == 0 {
        hlit -= 1;
    }
    
    // Trim trailing zeros for distance codes  
    while hdist > 0 && d_lengths[1 + hdist - 1] == 0 {
        hdist -= 1;
    }
    
    let hlit2 = hlit + 257;
    let lld_total = hlit2 + hdist + 1;
    
    // Rough estimation: 5 bits for HLIT + 5 bits for HDIST + 4 bits for HCLEN
    // Plus about 3 bits per code length on average
    14.0 + lld_total as f64 * 3.0
}

/// Calculate block size in bits for a specific block type
pub fn calculate_block_size(
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
    btype: i32
) -> f64 {
    let result = 3.0; // bfinal and btype bits
    
    match btype {
        0 => {
            // Uncompressed block
            let length = lz77.get_byte_range(lstart, lend);
            let rem = length % 65535;
            let blocks = length / 65535 + if rem != 0 { 1 } else { 0 };
            // Each uncompressed block header is 5 bytes: 3 bits, padding, LEN and NLEN
            (blocks * 5 * 8 + length * 8) as f64
        },
        1 => {
            // Fixed Huffman block
            let (ll_lengths, d_lengths) = get_fixed_tree();
            result + calculate_block_symbol_size(&ll_lengths, &d_lengths, lz77, lstart, lend) as f64
        },
        2 => {
            // Dynamic Huffman block
            let (tree_cost, _ll_lengths, _d_lengths) = get_dynamic_lengths(lz77, lstart, lend);
            result + tree_cost
        },
        _ => {
            panic!("Invalid block type: {}", btype);
        }
    }
}

/// Calculate block size automatically choosing the best block type
pub fn calculate_block_size_auto_type(
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize
) -> f64 {
    let uncompressed_cost = calculate_block_size(lz77, lstart, lend, 0);
    
    // Don't do the expensive fixed cost calculation for larger blocks that are
    // unlikely to use it.
    let fixed_cost = if lz77.size() > 1000 {
        uncompressed_cost
    } else {
        calculate_block_size(lz77, lstart, lend, 1)
    };
    
    let dynamic_cost = calculate_block_size(lz77, lstart, lend, 2);
    
    if uncompressed_cost < fixed_cost && uncompressed_cost < dynamic_cost {
        uncompressed_cost
    } else if fixed_cost < dynamic_cost {
        fixed_cost
    } else {
        dynamic_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::ZopfliOptions;
    use crate::lz77::{ZopfliBlockState, lz77_greedy};
    use crate::hash::ZopfliHash;

    #[test]
    fn test_bit_writer() {
        let mut writer = BitWriter::new();
        
        // Add some bits
        writer.add_bit(1);
        writer.add_bit(0);
        writer.add_bit(1);
        writer.add_bit(1);
        
        let data = writer.get_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], 0b00001101); // bits are added LSB first
        assert_eq!(writer.get_bit_pointer(), 4);
    }

    #[test]
    fn test_fixed_tree() {
        let (ll_lengths, d_lengths) = get_fixed_tree();
        
        // Check fixed tree structure
        assert_eq!(ll_lengths[0], 8);    // 0-143: length 8
        assert_eq!(ll_lengths[143], 8);
        assert_eq!(ll_lengths[144], 9);  // 144-255: length 9
        assert_eq!(ll_lengths[255], 9);
        assert_eq!(ll_lengths[256], 7);  // 256-279: length 7
        assert_eq!(ll_lengths[279], 7);
        assert_eq!(ll_lengths[280], 8);  // 280-287: length 8
        assert_eq!(ll_lengths[287], 8);
        
        // All distance codes have length 5
        for i in 0..32 {
            assert_eq!(d_lengths[i], 5);
        }
    }

    #[test]
    fn test_calculate_block_size_fixed() {
        let data = b"hello world";
        let mut store = ZopfliLZ77Store::new(data);
        let options = ZopfliOptions::default();
        let mut s = ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
        let mut hash = ZopfliHash::new(ZOPFLI_WINDOW_SIZE);
        
        // Create LZ77 representation
        lz77_greedy(&mut s, data, 0, data.len(), &mut store, &mut hash);
        
        // Calculate block size for fixed Huffman
        let size = calculate_block_size(&store, 0, store.size(), 1);
        
        // Should be reasonable (more than just the data, less than uncompressed)
        assert!(size > 0.0);
        assert!(size < (data.len() * 8) as f64);
    }

    #[test]
    fn test_calculate_block_size_auto_type() {
        let data = b"hello world hello world hello world";
        let mut store = ZopfliLZ77Store::new(data);
        let options = ZopfliOptions::default();
        let mut s = ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
        let mut hash = ZopfliHash::new(ZOPFLI_WINDOW_SIZE);
        
        lz77_greedy(&mut s, data, 0, data.len(), &mut store, &mut hash);
        
        let auto_size = calculate_block_size_auto_type(&store, 0, store.size());
        let uncompressed_size = calculate_block_size(&store, 0, store.size(), 0);
        let fixed_size = calculate_block_size(&store, 0, store.size(), 1);
        let dynamic_size = calculate_block_size(&store, 0, store.size(), 2);
        
        // Auto should pick the minimum
        assert!(auto_size <= uncompressed_size);
        assert!(auto_size <= fixed_size);
        assert!(auto_size <= dynamic_size);
        
        // Should pick one of the calculated sizes
        assert!(
            (auto_size - uncompressed_size).abs() < 1e-10 ||
            (auto_size - fixed_size).abs() < 1e-10 ||
            (auto_size - dynamic_size).abs() < 1e-10
        );
    }
}