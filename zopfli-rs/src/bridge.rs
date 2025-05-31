/// Get distance extra bits
pub fn get_dist_extra_bits(dist: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_dist_extra_bits(dist)
}

/// Get distance extra bits value
pub fn get_dist_extra_bits_value(dist: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_dist_extra_bits_value(dist)
}

/// Get distance symbol
pub fn get_dist_symbol(dist: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_dist_symbol(dist)
}

/// Get length extra bits
pub fn get_length_extra_bits(l: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_length_extra_bits(l)
}

/// Get length extra bits value
pub fn get_length_extra_bits_value(l: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_length_extra_bits_value(l)
}

/// Get length symbol
pub fn get_length_symbol(l: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_length_symbol(l)
}

/// Get length symbol extra bits
pub fn get_length_symbol_extra_bits(s: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_length_symbol_extra_bits(s)
}

/// Get distance symbol extra bits
pub fn get_dist_symbol_extra_bits(s: i32) -> i32 {
    // Always use Rust implementation
    crate::symbols::get_dist_symbol_extra_bits(s)
}

/// Initialize ZopfliOptions with default values
/// This bridges between C ZopfliInitOptions and Rust Default::default()
pub fn init_options() -> crate::options::ZopfliOptions {
    // Always use Rust implementation
    crate::options::ZopfliOptions::default()
}

/// Calculate length-limited code lengths using Katajainen algorithm
pub fn length_limited_code_lengths(
    frequencies: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    // Always use Rust implementation
    crate::tree::length_limited_code_lengths(frequencies, maxbits, bitlengths)
}

/// Calculate bit lengths for Huffman tree
pub fn calculate_bit_lengths(
    count: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    // Always use Rust implementation
    crate::tree::calculate_bit_lengths(count, maxbits, bitlengths)
}

/// Convert Huffman code lengths to symbols
pub fn lengths_to_symbols(
    lengths: &[u32],
    maxbits: u32,
    symbols: &mut [u32],
) {
    // Always use Rust implementation
    crate::tree::lengths_to_symbols(lengths, maxbits, symbols)
}

/// Calculate entropy for each symbol
pub fn calculate_entropy(count: &[usize], bitlengths: &mut [f64]) {
    // Always use Rust implementation
    crate::tree::calculate_entropy(count, bitlengths)
}

/// Bridge for ZopfliHash - provides unified interface for C and Rust implementations
pub struct ZopfliHashBridge {
    // Always use Rust implementation
    rust_hash: crate::hash::ZopfliHash,
}

impl ZopfliHashBridge {
    pub fn new(window_size: usize) -> Self {
        // Always use Rust implementation
        ZopfliHashBridge {
            rust_hash: crate::hash::ZopfliHash::new(window_size),
        }
    }
    
    pub fn update(&mut self, array: &[u8], pos: usize, end: usize) {
        // Always use Rust implementation
        self.rust_hash.update(array, pos, end);
    }
    
    pub fn warmup(&mut self, array: &[u8], pos: usize, end: usize) {
        // Always use Rust implementation
        self.rust_hash.warmup(array, pos, end);
    }
}

// Drop is automatic for Rust implementation

/// Bridge for ZopfliLongestMatchCache - provides unified interface for C and Rust implementations
pub struct ZopfliLongestMatchCacheBridge {
    // Always use Rust implementation
    rust_cache: crate::cache::ZopfliLongestMatchCache,
}

impl ZopfliLongestMatchCacheBridge {
    pub fn new(blocksize: usize) -> Result<Self, String> {
        // Always use Rust implementation
        let rust_cache = crate::cache::ZopfliLongestMatchCache::new(blocksize)?;
        Ok(ZopfliLongestMatchCacheBridge { rust_cache })
    }
    
    pub fn sublen_to_cache(&mut self, sublen: &[u16], pos: usize, length: usize) {
        // Always use Rust implementation
        self.rust_cache.sublen_to_cache(sublen, pos, length);
    }
    
    pub fn cache_to_sublen(&self, pos: usize, length: usize, sublen: &mut [u16]) {
        // Always use Rust implementation
        self.rust_cache.cache_to_sublen(pos, length, sublen);
    }
    
    pub fn max_cached_sublen(&self, pos: usize, length: usize) -> usize {
        // Always use Rust implementation
        self.rust_cache.max_cached_sublen(pos, length)
    }
}

// Drop is automatic for Rust implementation

/// LZ77 optimal compression
pub fn lz77_optimal<'a>(
    s: &mut crate::lz77::ZopfliBlockState,
    input: &'a [u8],
    instart: usize,
    inend: usize,
    numiterations: i32,
    store: &mut crate::lz77::ZopfliLZ77Store<'a>,
) {
    // Always use Rust implementation
    crate::squeeze::lz77_optimal(s, input, instart, inend, numiterations, store)
}

/// LZ77 optimal fixed compression
pub fn lz77_optimal_fixed<'a>(
    s: &mut crate::lz77::ZopfliBlockState,
    input: &'a [u8],
    instart: usize,
    inend: usize,
    store: &mut crate::lz77::ZopfliLZ77Store<'a>,
) {
    // Always use Rust implementation
    crate::squeeze::lz77_optimal_fixed(s, input, instart, inend, store)
}

// lz77_store_to_c function removed - no longer needed since bridge always uses Rust

/// Block splitting functions

/// Does blocksplitting on LZ77 data.
/// The output splitpoints are indices in the LZ77 data.
/// maxblocks: set a limit to the amount of blocks. Set to 0 to mean no limit.
pub fn block_split_lz77(
    options: &crate::options::ZopfliOptions,
    lz77: &crate::lz77::ZopfliLZ77Store,
    maxblocks: usize,
) -> Vec<usize> {
    // Always use Rust implementation
    crate::blocksplitter::block_split_lz77(options, lz77, maxblocks)
}

/// Does blocksplitting on uncompressed data.
/// The output splitpoints are indices in the uncompressed bytes.
pub fn block_split(
    options: &crate::options::ZopfliOptions,
    input: &[u8],
    instart: usize,
    inend: usize,
    maxblocks: usize,
) -> Vec<usize> {
    // Always use Rust implementation
    crate::blocksplitter::block_split(options, input, instart, inend, maxblocks)
}

/// Divides the input into equal blocks, does not even take LZ77 lengths into
/// account.
pub fn block_split_simple(
    input: &[u8],
    instart: usize,
    inend: usize,
    blocksize: usize,
) -> Vec<usize> {
    // Always use Rust implementation
    crate::blocksplitter::block_split_simple(input, instart, inend, blocksize)
}

/// Calculate block size in bits for a specific block type
pub fn calculate_block_size(
    lz77: &crate::lz77::ZopfliLZ77Store,
    lstart: usize,
    lend: usize,
    btype: i32
) -> f64 {
    // Always use Rust implementation
    crate::deflate::calculate_block_size(lz77, lstart, lend, btype)
}

/// Calculate block size automatically choosing the best block type
pub fn calculate_block_size_auto_type(
    lz77: &crate::lz77::ZopfliLZ77Store,
    lstart: usize,
    lend: usize
) -> f64 {
    // Always use Rust implementation
    crate::deflate::calculate_block_size_auto_type(lz77, lstart, lend)
}