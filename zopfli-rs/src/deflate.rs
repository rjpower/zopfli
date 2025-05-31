use crate::util::*;
use crate::symbols::*;
use crate::tree::{calculate_bit_lengths, lengths_to_symbols};
use crate::lz77::ZopfliLZ77Store;
use crate::options::ZopfliOptions;

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
pub fn patch_distance_codes_for_buggy_decoders(d_lengths: &mut [u32; ZOPFLI_NUM_D]) {
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
pub fn calculate_block_symbol_size_given_counts(
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
    
    // Try with and without RLE optimization
    let tree_size = calculate_tree_size(&ll_lengths, &d_lengths);
    let data_size = calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, &ll_lengths, &d_lengths, lz77, lstart, lend);
    
    // Try RLE optimization
    let mut ll_counts2 = ll_counts.clone();
    let mut d_counts2 = d_counts.clone();
    optimize_huffman_for_rle(ZOPFLI_NUM_LL, &mut ll_counts2);
    optimize_huffman_for_rle(ZOPFLI_NUM_D, &mut d_counts2);
    
    let mut ll_lengths2 = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths2 = [0u32; ZOPFLI_NUM_D];
    let _ = calculate_bit_lengths(&ll_counts2, 15, &mut ll_lengths2);
    let _ = calculate_bit_lengths(&d_counts2, 15, &mut d_lengths2);
    patch_distance_codes_for_buggy_decoders(&mut d_lengths2);
    
    let tree_size2 = calculate_tree_size(&ll_lengths2, &d_lengths2);
    let data_size2 = calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, &ll_lengths2, &d_lengths2, lz77, lstart, lend);
    
    // Choose the better option
    if tree_size2 + (data_size2 as f64) < tree_size + (data_size as f64) {
        (tree_size2 + data_size2 as f64, ll_lengths2, d_lengths2)
    } else {
        (tree_size + data_size as f64, ll_lengths, d_lengths)
    }
}

/// Optimize Huffman tree for RLE encoding
pub fn optimize_huffman_for_rle(length: usize, counts: &mut [usize]) {
    // 1) Don't touch trailing zeros
    let mut actual_length = length;
    while actual_length > 0 {
        if counts[actual_length - 1] != 0 {
            break;
        }
        actual_length -= 1;
    }
    if actual_length == 0 {
        return;
    }
    
    // 2) Mark all population counts that already can be encoded with an rle code
    let mut good_for_rle = vec![false; actual_length];
    
    // Mark any seq of 0's that is longer than 5 as good_for_rle
    // Mark any seq of non-0's that is longer than 7 as good_for_rle
    let mut symbol = counts[0];
    let mut stride = 0;
    for i in 0..=actual_length {
        if i == actual_length || counts[i] != symbol {
            if (symbol == 0 && stride >= 5) || (symbol != 0 && stride >= 7) {
                for k in 0..stride {
                    good_for_rle[i - k - 1] = true;
                }
            }
            stride = 1;
            if i != actual_length {
                symbol = counts[i];
            }
        } else {
            stride += 1;
        }
    }
    
    // 3) Replace population counts that lead to more rle codes
    let mut stride = 0;
    let mut limit = counts[0];
    let mut sum = 0;
    for i in 0..=actual_length {
        if i == actual_length || good_for_rle[i] || counts[i].abs_diff(limit) >= 4 {
            if stride >= 4 || (stride >= 3 && sum == 0) {
                let count = if sum == 0 {
                    0
                } else {
                    ((sum + stride / 2) / stride).max(1)
                };
                for k in 0..stride {
                    counts[i - k - 1] = count;
                }
            }
            stride = 0;
            sum = 0;
            if i + 3 < actual_length {
                limit = (counts[i] + counts[i + 1] + counts[i + 2] + counts[i + 3] + 2) / 4;
            } else if i < actual_length {
                limit = counts[i];
            } else {
                limit = 0;
            }
        }
        stride += 1;
        if i != actual_length {
            sum += counts[i];
        }
    }
}

/// Encode tree and calculate its size
fn encode_tree(
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    use_16: bool,
    use_17: bool,
    use_18: bool,
) -> f64 {
    let mut hlit = 29;
    let mut hdist = 29;
    
    // Trim zeros
    while hlit > 0 && ll_lengths[257 + hlit - 1] == 0 {
        hlit -= 1;
    }
    while hdist > 0 && d_lengths[1 + hdist - 1] == 0 {
        hdist -= 1;
    }
    
    let hlit2 = hlit + 257;
    let lld_total = hlit2 + hdist + 1;
    
    // Create combined array of code lengths
    let mut lld = vec![0u32; lld_total];
    for i in 0..hlit2 {
        lld[i] = ll_lengths[i];
    }
    for i in 0..=hdist {
        lld[hlit2 + i] = d_lengths[i];
    }
    
    // RLE encode the lengths
    let mut rle = Vec::new();
    let mut rle_bits = Vec::new();
    let mut i = 0;
    while i < lld_total {
        let value = lld[i];
        let mut j = i + 1;
        
        // Count consecutive values
        while j < lld_total && lld[j] == value {
            j += 1;
        }
        let count = j - i;
        
        if value == 0 && count >= 3 {
            // Use codes 17 or 18 for runs of zeros
            let mut remaining = count;
            while remaining >= 11 && use_18 {
                let run_len = remaining.min(138);
                rle.push(18);
                rle_bits.push((run_len - 11) as u32);
                remaining -= run_len;
            }
            while remaining >= 3 && use_17 {
                let run_len = remaining.min(10);
                rle.push(17);
                rle_bits.push((run_len - 3) as u32);
                remaining -= run_len;
            }
            // Output remaining zeros directly
            for _ in 0..remaining {
                rle.push(0);
                rle_bits.push(0);
            }
        } else if value != 0 && count >= 4 {
            // Use code 16 for runs of non-zero values
            rle.push(value);
            rle_bits.push(0);
            let mut remaining = count - 1;
            while remaining >= 3 && use_16 {
                let run_len = remaining.min(6);
                rle.push(16);
                rle_bits.push((run_len - 3) as u32);
                remaining -= run_len;
            }
            // Output remaining values directly
            for _ in 0..remaining {
                rle.push(value);
                rle_bits.push(0);
            }
        } else {
            // Output values directly
            for _ in 0..count {
                rle.push(value);
                rle_bits.push(0);
            }
        }
        
        i = j;
    }
    
    // Count code length code frequencies
    let mut clcounts = [0usize; 19];
    for &symbol in &rle {
        clcounts[symbol as usize] += 1;
    }
    
    let mut clcl = [0u32; 19];
    let _ = calculate_bit_lengths(&clcounts, 7, &mut clcl);
    
    // Trim trailing zeros from clcl
    let order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut hclen = 15;
    while hclen > 0 && clcl[order[hclen + 3]] == 0 {
        hclen -= 1;
    }
    
    // Calculate size
    let mut result = 14.0; // 5 + 5 + 4 bits for HLIT, HDIST, HCLEN
    result += (hclen + 4) as f64 * 3.0; // Code length code lengths
    
    // Size of RLE encoded data
    for &symbol in &rle {
        result += clcl[symbol as usize] as f64;
        match symbol {
            16 => result += 2.0, // 2 extra bits
            17 => result += 3.0, // 3 extra bits
            18 => result += 7.0, // 7 extra bits
            _ => {}
        }
    }
    
    result
}

/// Calculate tree size trying all RLE encoding options
pub fn calculate_tree_size(ll_lengths: &[u32; ZOPFLI_NUM_LL], d_lengths: &[u32; ZOPFLI_NUM_D]) -> f64 {
    let mut best_size = f64::MAX;
    
    // Try all 8 combinations of using codes 16, 17, 18
    for i in 0..8 {
        let use_16 = (i & 1) != 0;
        let use_17 = (i & 2) != 0;
        let use_18 = (i & 4) != 0;
        
        let size = encode_tree(ll_lengths, d_lengths, use_16, use_17, use_18);
        if size < best_size {
            best_size = size;
        }
    }
    
    best_size
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

/// Add the optimal dynamic Huffman tree to the output
pub fn add_dynamic_tree(
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    writer: &mut BitWriter,
) {
    let mut best_size = f64::MAX;
    let mut best_flags = 0;
    
    // Try all 8 combinations to find the best encoding
    for i in 0..8 {
        let size = encode_tree(ll_lengths, d_lengths, i & 1 != 0, i & 2 != 0, i & 4 != 0);
        if size < best_size {
            best_size = size;
            best_flags = i;
        }
    }
    
    // Actually encode the tree with the best parameters
    encode_tree_with_output(
        ll_lengths,
        d_lengths,
        best_flags & 1 != 0,
        best_flags & 2 != 0,
        best_flags & 4 != 0,
        writer,
    );
}

/// Encode tree with actual output
fn encode_tree_with_output(
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    use_16: bool,
    use_17: bool,
    use_18: bool,
    writer: &mut BitWriter,
) {
    let mut hlit = 29;
    let mut hdist = 29;
    
    // Trim zeros
    while hlit > 0 && ll_lengths[257 + hlit - 1] == 0 {
        hlit -= 1;
    }
    while hdist > 0 && d_lengths[1 + hdist - 1] == 0 {
        hdist -= 1;
    }
    
    let hlit2 = hlit + 257;
    let lld_total = hlit2 + hdist + 1;
    
    // Create combined array of code lengths
    let mut lld = vec![0u32; lld_total];
    for i in 0..hlit2 {
        lld[i] = ll_lengths[i];
    }
    for i in 0..=hdist {
        lld[hlit2 + i] = d_lengths[i];
    }
    
    // RLE encode the lengths
    let mut rle = Vec::new();
    let mut rle_bits = Vec::new();
    let mut i = 0;
    while i < lld_total {
        let value = lld[i];
        let mut j = i + 1;
        
        // Count consecutive values
        while j < lld_total && lld[j] == value {
            j += 1;
        }
        let count = j - i;
        
        if value == 0 && count >= 3 {
            // Use codes 17 or 18 for runs of zeros
            let mut remaining = count;
            while remaining >= 11 && use_18 {
                let run_len = remaining.min(138);
                rle.push(18);
                rle_bits.push((run_len - 11) as u32);
                remaining -= run_len;
            }
            while remaining >= 3 && use_17 {
                let run_len = remaining.min(10);
                rle.push(17);
                rle_bits.push((run_len - 3) as u32);
                remaining -= run_len;
            }
            // Output remaining zeros directly
            for _ in 0..remaining {
                rle.push(0);
                rle_bits.push(0);
            }
        } else if value != 0 && count >= 4 {
            // Use code 16 for runs of non-zero values
            rle.push(value);
            rle_bits.push(0);
            let mut remaining = count - 1;
            while remaining >= 3 && use_16 {
                let run_len = remaining.min(6);
                rle.push(16);
                rle_bits.push((run_len - 3) as u32);
                remaining -= run_len;
            }
            // Output remaining values directly
            for _ in 0..remaining {
                rle.push(value);
                rle_bits.push(0);
            }
        } else {
            // Output values directly
            for _ in 0..count {
                rle.push(value);
                rle_bits.push(0);
            }
        }
        
        i = j;
    }
    
    // Count code length code frequencies
    let mut clcounts = [0usize; 19];
    for &symbol in &rle {
        clcounts[symbol as usize] += 1;
    }
    
    let mut clcl = [0u32; 19];
    let _ = calculate_bit_lengths(&clcounts, 7, &mut clcl);
    
    let mut clsymbols = [0u32; 19];
    lengths_to_symbols(&clcl, 19, &mut clsymbols);
    
    // Trim trailing zeros from clcl
    let order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut hclen = 15;
    while hclen > 0 && clcl[order[hclen + 3]] == 0 {
        hclen -= 1;
    }
    
    // Write the header
    writer.add_bits(hlit as u32, 5);
    writer.add_bits(hdist as u32, 5);
    writer.add_bits(hclen as u32, 4);
    
    // Write the code length codes
    for i in 0..hclen + 4 {
        writer.add_bits(clcl[order[i]], 3);
    }
    
    // Write the RLE encoded data
    for i in 0..rle.len() {
        let symbol = clsymbols[rle[i] as usize];
        writer.add_huffman_bits(symbol, clcl[rle[i] as usize]);
        
        // Extra bits
        match rle[i] {
            16 => writer.add_bits(rle_bits[i], 2),
            17 => writer.add_bits(rle_bits[i], 3),
            18 => writer.add_bits(rle_bits[i], 7),
            _ => {}
        }
    }
}

/// Add LZ77 data to the output
pub fn add_lz77_data(
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
    expected_data_size: usize,
    ll_symbols: &[u32; ZOPFLI_NUM_LL],
    ll_lengths: &[u32; ZOPFLI_NUM_LL],
    d_symbols: &[u32; ZOPFLI_NUM_D],
    d_lengths: &[u32; ZOPFLI_NUM_D],
    writer: &mut BitWriter,
) {
    let mut test_length = 0;
    
    for i in lstart..lend {
        let dist = lz77.dists()[i];
        let litlen = lz77.litlens()[i];
        
        if dist == 0 {
            // Literal
            assert!(litlen < 256);
            assert!(ll_lengths[litlen as usize] > 0);
            writer.add_huffman_bits(ll_symbols[litlen as usize], ll_lengths[litlen as usize]);
            test_length += 1;
        } else {
            // Length-distance pair
            let lls = get_length_symbol(litlen as i32) as usize;
            let ds = get_dist_symbol(dist as i32) as usize;
            assert!(litlen >= 3 && litlen <= 288);
            assert!(ll_lengths[lls] > 0);
            assert!(d_lengths[ds] > 0);
            
            writer.add_huffman_bits(ll_symbols[lls], ll_lengths[lls]);
            writer.add_bits(
                get_length_extra_bits_value(litlen as i32) as u32,
                get_length_extra_bits(litlen as i32) as u32,
            );
            writer.add_huffman_bits(d_symbols[ds], d_lengths[ds]);
            writer.add_bits(
                get_dist_extra_bits_value(dist as i32) as u32,
                get_dist_extra_bits(dist as i32) as u32,
            );
            test_length += litlen as usize;
        }
    }
    
    assert!(expected_data_size == 0 || test_length == expected_data_size);
}

/// Add a non-compressed block to the output
pub fn add_non_compressed_block(
    final_block: bool,
    input: &[u8],
    instart: usize,
    inend: usize,
    writer: &mut BitWriter,
) {
    let blocksize = 65535;
    let mut pos = instart;
    
    while pos < inend {
        let current_block_size = (inend - pos).min(blocksize);
        let is_final = final_block && pos + current_block_size >= inend;
        
        // Write block header
        writer.add_bit(if is_final { 1 } else { 0 }); // BFINAL
        writer.add_bits(0, 2); // BTYPE = 00
        
        // Pad to byte boundary
        if writer.bit_pointer != 0 {
            writer.add_bits(0, (8 - writer.bit_pointer) as u32);
        }
        
        // Write LEN and NLEN
        writer.add_bits(current_block_size as u32, 16);
        writer.add_bits(!current_block_size as u32, 16);
        
        // Write the data
        for i in 0..current_block_size {
            writer.add_bits(input[pos + i] as u32, 8);
        }
        
        pos += current_block_size;
    }
}

/// Add an LZ77 compressed block to the output
pub fn add_lz77_block(
    btype: i32,
    final_block: bool,
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
    expected_data_size: usize,
    writer: &mut BitWriter,
) {
    // Write block header
    writer.add_bit(if final_block { 1 } else { 0 }); // BFINAL
    writer.add_bits(btype as u32, 2); // BTYPE
    
    let mut ll_lengths = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths = [0u32; ZOPFLI_NUM_D];
    let mut ll_symbols = [0u32; ZOPFLI_NUM_LL];
    let mut d_symbols = [0u32; ZOPFLI_NUM_D];
    
    if btype == 1 {
        // Fixed Huffman codes
        let (fixed_ll, fixed_d) = get_fixed_tree();
        ll_lengths = fixed_ll;
        d_lengths = fixed_d;
    } else {
        // Dynamic Huffman codes
        let (_cost, dynamic_ll, dynamic_d) = get_dynamic_lengths(lz77, lstart, lend);
        ll_lengths = dynamic_ll;
        d_lengths = dynamic_d;
        
        // Add the tree to the output
        add_dynamic_tree(&ll_lengths, &d_lengths, writer);
    }
    
    // Convert lengths to symbols
    lengths_to_symbols(&ll_lengths, 15, &mut ll_symbols);
    lengths_to_symbols(&d_lengths, 15, &mut d_symbols);
    
    // Add the compressed data
    add_lz77_data(
        lz77,
        lstart,
        lend,
        expected_data_size,
        &ll_symbols,
        &ll_lengths,
        &d_symbols,
        &d_lengths,
        writer,
    );
    
    // Add end symbol
    writer.add_huffman_bits(ll_symbols[256], ll_lengths[256]);
}

/// Add an LZ77 block with automatic block type selection
pub fn add_lz77_block_auto_type(
    options: &ZopfliOptions,
    final_block: bool,
    lz77: &ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
    expected_data_size: usize,
    writer: &mut BitWriter,
) {
    use crate::squeeze::lz77_optimal_fixed;
    
    let uncompressed_cost = calculate_block_size(lz77, lstart, lend, 0);
    let fixed_cost = calculate_block_size(lz77, lstart, lend, 1);
    let dynamic_cost = calculate_block_size(lz77, lstart, lend, 2);
    
    // Whether to perform the expensive calculation of creating an optimal block
    // with fixed Huffman tree to check if smaller
    let expensive_fixed = lz77.size() < 1000 || fixed_cost <= dynamic_cost * 1.1;
    
    if lstart == lend {
        // Empty block - use fixed type
        writer.add_bit(if final_block { 1 } else { 0 });
        writer.add_bits(1, 2); // BTYPE = 01
        writer.add_bits(0, 7); // End symbol has code 0000000
        return;
    }
    
    let mut fixed_store = None;
    let mut actual_fixed_cost = fixed_cost;
    
    if expensive_fixed {
        // Recalculate the LZ77 with ZopfliLZ77OptimalFixed
        let instart = lz77.pos()[lstart];
        let inend = instart + lz77.get_byte_range(lstart, lend);
        
        let mut s = crate::lz77::ZopfliBlockState::new(options, instart, inend, true).unwrap();
        let mut store = ZopfliLZ77Store::new(lz77.data());
        lz77_optimal_fixed(&mut s, lz77.data(), instart, inend, &mut store);
        actual_fixed_cost = calculate_block_size(&store, 0, store.size(), 1);
        fixed_store = Some(store);
    }
    
    if uncompressed_cost < actual_fixed_cost && uncompressed_cost < dynamic_cost {
        // Uncompressed block
        let instart = lz77.pos()[lstart];
        let inend = instart + lz77.get_byte_range(lstart, lend);
        add_non_compressed_block(final_block, lz77.data(), instart, inend, writer);
    } else if actual_fixed_cost < dynamic_cost {
        // Fixed Huffman block
        if let Some(store) = fixed_store {
            add_lz77_block(1, final_block, &store, 0, store.size(), expected_data_size, writer);
        } else {
            add_lz77_block(1, final_block, lz77, lstart, lend, expected_data_size, writer);
        }
    } else {
        // Dynamic Huffman block
        add_lz77_block(2, final_block, lz77, lstart, lend, expected_data_size, writer);
    }
}

/// Deflate a part of the input data
pub fn deflate_part(
    options: &ZopfliOptions,
    btype: i32,
    final_block: bool,
    input: &[u8],
    instart: usize,
    inend: usize,
    writer: &mut BitWriter,
) {
    use crate::blocksplitter::{block_split, block_split_lz77};
    use crate::lz77::ZopfliBlockState;
    use crate::squeeze::{lz77_optimal, lz77_optimal_fixed};
    
    // If btype=2 is specified, it tries all block types. If a lesser btype is
    // given, then however it forces that one. Neither of the lesser types needs
    // block splitting as they have no dynamic huffman trees.
    if btype == 0 {
        add_non_compressed_block(final_block, input, instart, inend, writer);
        return;
    } else if btype == 1 {
        let mut store = ZopfliLZ77Store::new(input);
        let mut s = ZopfliBlockState::new(options, instart, inend, true).unwrap();
        
        lz77_optimal_fixed(&mut s, input, instart, inend, &mut store);
        add_lz77_block(1, final_block, &store, 0, store.size(), 0, writer);
        
        return;
    }
    
    // Block splitting for btype 2
    let mut splitpoints = Vec::new();
    
    if options.blocksplitting != 0 {
        splitpoints = block_split(options, input, instart, inend, options.blocksplittingmax as usize);
    }
    
    let mut lz77 = ZopfliLZ77Store::new(input);
    let mut totalcost = 0.0;
    
    // Create optimal LZ77 encoding for each block
    for i in 0..=splitpoints.len() {
        let start = if i == 0 { instart } else { splitpoints[i - 1] };
        let end = if i == splitpoints.len() { inend } else { splitpoints[i] };
        
        let mut s = ZopfliBlockState::new(options, start, end, true).unwrap();
        let mut store = ZopfliLZ77Store::new(input);
        lz77_optimal(&mut s, input, start, end, options.numiterations, &mut store);
        totalcost += calculate_block_size_auto_type(&store, 0, store.size());
        
        lz77.append_store(&store);
    }
    
    // Convert splitpoints from byte positions to LZ77 positions
    let mut lz77_splitpoints = Vec::new();
    for i in 0..splitpoints.len() {
        lz77_splitpoints.push(lz77.size()); // This is a simplification; actual implementation needs proper mapping
    }
    
    // Second block splitting attempt on the LZ77 data
    if options.blocksplitting != 0 && splitpoints.len() > 1 {
        let mut splitpoints2 = block_split_lz77(options, &lz77, options.blocksplittingmax as usize);
        
        let mut totalcost2 = 0.0;
        for i in 0..=splitpoints2.len() {
            let start = if i == 0 { 0 } else { splitpoints2[i - 1] };
            let end = if i == splitpoints2.len() { lz77.size() } else { splitpoints2[i] };
            totalcost2 += calculate_block_size_auto_type(&lz77, start, end);
        }
        
        if totalcost2 < totalcost {
            lz77_splitpoints = splitpoints2;
        }
    }
    
    // Output the blocks
    for i in 0..=lz77_splitpoints.len() {
        let start = if i == 0 { 0 } else { lz77_splitpoints[i - 1] };
        let end = if i == lz77_splitpoints.len() { lz77.size() } else { lz77_splitpoints[i] };
        let is_final = i == lz77_splitpoints.len() && final_block;
        
        add_lz77_block_auto_type(options, is_final, &lz77, start, end, 0, writer);
    }
}

/// Main DEFLATE compression function
pub fn deflate(
    options: &ZopfliOptions,
    btype: i32,
    final_block: bool,
    input: &[u8],
    insize: usize,
    writer: &mut BitWriter,
) {
    let offset = writer.data.len();
    
    if ZOPFLI_MASTER_BLOCK_SIZE == 0 {
        deflate_part(options, btype, final_block, input, 0, insize, writer);
    } else {
        let mut i = 0;
        while i < insize {
            let master_final = i + ZOPFLI_MASTER_BLOCK_SIZE >= insize;
            let final2 = final_block && master_final;
            let size = if master_final { insize - i } else { ZOPFLI_MASTER_BLOCK_SIZE };
            deflate_part(options, btype, final2, input, i, i + size, writer);
            i += size;
        }
    }
    
    if options.verbose != 0 {
        eprintln!(
            "Original Size: {}, Deflate: {}, Compression: {:.1}% Removed",
            insize,
            writer.data.len() - offset,
            100.0 * (insize as f64 - (writer.data.len() - offset) as f64) / insize as f64
        );
    }
}