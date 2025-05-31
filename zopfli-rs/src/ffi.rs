use std::os::raw::{c_int, c_double, c_uchar, c_ushort};
use crate::options::ZopfliOptions;

// C struct definitions for FFI
// Based on CODEBASE_ANALYSIS.md, all conditional compilation flags are assumed active
#[repr(C)]
pub struct ZopfliHashC {
    head: *mut c_int,
    prev: *mut u16,
    hashval: *mut c_int,
    val: c_int,
    // ZOPFLI_HASH_SAME_HASH fields
    head2: *mut c_int,
    prev2: *mut u16,
    hashval2: *mut c_int,
    val2: c_int,
    // ZOPFLI_HASH_SAME fields
    same: *mut u16,
}

#[repr(C)]
pub struct ZopfliLongestMatchCacheC {
    length: *mut u16,
    dist: *mut u16,
    sublen: *mut u8,
}

#[repr(C)]
pub struct ZopfliBlockStateC {
    options: *const ZopfliOptions,
    // ZOPFLI_LONGEST_MATCH_CACHE field
    lmc: *mut ZopfliLongestMatchCacheC,
    blockstart: usize,
    blockend: usize,
}

#[repr(C)]
pub struct ZopfliLZ77StoreC {
    litlens: *mut c_ushort,
    dists: *mut c_ushort,
    size: usize,
    data: *const c_uchar,
    pos: *mut usize,
    ll_symbol: *mut c_ushort,
    d_symbol: *mut c_ushort,
    ll_counts: *mut usize,
    d_counts: *mut usize,
}

#[cfg(feature = "c-fallback")]
extern "C" {
    // Options functions
    pub fn ZopfliInitOptions(options: *mut ZopfliOptions);

    // Symbol functions - using wrapper functions since the originals are static inline
    pub fn zopfli_get_dist_extra_bits_wrapper(dist: c_int) -> c_int;
    pub fn zopfli_get_dist_extra_bits_value_wrapper(dist: c_int) -> c_int;
    pub fn zopfli_get_dist_symbol_wrapper(dist: c_int) -> c_int;
    pub fn zopfli_get_length_extra_bits_wrapper(l: c_int) -> c_int;
    pub fn zopfli_get_length_extra_bits_value_wrapper(l: c_int) -> c_int;
    pub fn zopfli_get_length_symbol_wrapper(l: c_int) -> c_int;
    pub fn zopfli_get_length_symbol_extra_bits_wrapper(s: c_int) -> c_int;
    pub fn zopfli_get_dist_symbol_extra_bits_wrapper(s: c_int) -> c_int;

    // Tree functions - these are regular exported functions, no wrappers needed
    pub fn ZopfliLengthLimitedCodeLengths(
        frequencies: *const usize, 
        n: c_int, 
        maxbits: c_int, 
        bitlengths: *mut u32
    ) -> c_int;
    
    pub fn ZopfliCalculateBitLengths(
        count: *const usize, 
        n: usize, 
        maxbits: c_int,
        bitlengths: *mut u32
    );
    
    pub fn ZopfliLengthsToSymbols(
        lengths: *const u32, 
        n: usize, 
        maxbits: u32,
        symbols: *mut u32
    );
    
    pub fn ZopfliCalculateEntropy(
        count: *const usize, 
        n: usize, 
        bitlengths: *mut c_double
    );

    // Hash functions - these are regular exported functions
    pub fn ZopfliAllocHash(window_size: usize, h: *mut ZopfliHashC);
    pub fn ZopfliResetHash(window_size: usize, h: *mut ZopfliHashC);
    pub fn ZopfliCleanHash(h: *mut ZopfliHashC);
    pub fn ZopfliUpdateHash(array: *const u8, pos: usize, end: usize, h: *mut ZopfliHashC);
    pub fn ZopfliWarmupHash(array: *const u8, pos: usize, end: usize, h: *mut ZopfliHashC);

    // Cache functions - these are regular exported functions  
    pub fn ZopfliInitCache(blocksize: usize, lmc: *mut ZopfliLongestMatchCacheC);
    pub fn ZopfliCleanCache(lmc: *mut ZopfliLongestMatchCacheC);
    pub fn ZopfliSublenToCache(sublen: *const u16, pos: usize, length: usize, lmc: *mut ZopfliLongestMatchCacheC);
    pub fn ZopfliCacheToSublen(lmc: *const ZopfliLongestMatchCacheC, pos: usize, length: usize, sublen: *mut u16);
    pub fn ZopfliMaxCachedSublen(lmc: *const ZopfliLongestMatchCacheC, pos: usize, length: usize) -> u32;

    // BlockState functions
    pub fn ZopfliInitBlockState(
        options: *const ZopfliOptions,
        blockstart: usize,
        blockend: usize,
        add_lmc: c_int,
        s: *mut ZopfliBlockStateC
    );
    
    pub fn ZopfliCleanBlockState(s: *mut ZopfliBlockStateC);
    
    // LZ77 functions - the actual C implementation
    pub fn ZopfliFindLongestMatch(
        s: *mut ZopfliBlockStateC,
        h: *const ZopfliHashC,
        array: *const c_uchar,
        pos: usize,
        size: usize,
        limit: usize,
        sublen: *mut c_ushort,
        distance: *mut c_ushort,
        length: *mut c_ushort
    );
    
    pub fn ZopfliVerifyLenDist(
        data: *const c_uchar,
        datasize: usize,
        pos: usize,
        dist: c_ushort,
        length: c_ushort
    );

    // LZ77Store functions
    pub fn ZopfliInitLZ77Store(data: *const c_uchar, store: *mut ZopfliLZ77StoreC);
    pub fn ZopfliCleanLZ77Store(store: *mut ZopfliLZ77StoreC);
    pub fn ZopfliCopyLZ77Store(source: *const ZopfliLZ77StoreC, dest: *mut ZopfliLZ77StoreC);
    pub fn ZopfliStoreLitLenDist(length: c_ushort, dist: c_ushort, pos: usize, store: *mut ZopfliLZ77StoreC);
    pub fn ZopfliAppendLZ77Store(store: *const ZopfliLZ77StoreC, target: *mut ZopfliLZ77StoreC);
    pub fn ZopfliLZ77GetByteRange(lz77: *const ZopfliLZ77StoreC, lstart: usize, lend: usize) -> usize;
    pub fn ZopfliLZ77GetHistogram(
        lz77: *const ZopfliLZ77StoreC,
        lstart: usize,
        lend: usize,
        ll_counts: *mut usize,
        d_counts: *mut usize
    );
    pub fn ZopfliLZ77Greedy(
        s: *mut ZopfliBlockStateC,
        input: *const c_uchar,
        instart: usize,
        inend: usize,
        store: *mut ZopfliLZ77StoreC,
        h: *mut ZopfliHashC
    );

    // Squeeze functions - the main external functions we're porting
    pub fn ZopfliLZ77Optimal(
        s: *mut ZopfliBlockStateC,
        input: *const c_uchar,
        instart: usize,
        inend: usize,
        numiterations: c_int,
        store: *mut ZopfliLZ77StoreC
    );

    pub fn ZopfliLZ77OptimalFixed(
        s: *mut ZopfliBlockStateC,
        input: *const c_uchar,
        instart: usize,
        inend: usize,
        store: *mut ZopfliLZ77StoreC
    );

    // Calculate block size functions (from deflate.h) - needed for cost calculation
    pub fn ZopfliCalculateBlockSize(
        lz77: *const ZopfliLZ77StoreC,
        lstart: usize,
        lend: usize,
        btype: c_int
    ) -> c_double;

    // Helper functions for testing LZ77Store contents
    pub fn ZopfliLZ77StoreGetSize(store: *const ZopfliLZ77StoreC) -> usize;
    pub fn ZopfliLZ77StoreGetLitLen(store: *const ZopfliLZ77StoreC, index: usize) -> c_ushort;
    pub fn ZopfliLZ77StoreGetDist(store: *const ZopfliLZ77StoreC, index: usize) -> c_ushort;
    pub fn ZopfliLZ77StoreGetPos(store: *const ZopfliLZ77StoreC, index: usize) -> usize;

}

// Convenience wrappers for the symbol functions
#[cfg(feature = "c-fallback")]
pub mod symbols {
    use super::*;
    
    #[inline]
    pub unsafe fn get_dist_extra_bits(dist: c_int) -> c_int {
        zopfli_get_dist_extra_bits_wrapper(dist)
    }
    
    #[inline]
    pub unsafe fn get_dist_extra_bits_value(dist: c_int) -> c_int {
        zopfli_get_dist_extra_bits_value_wrapper(dist)
    }
    
    #[inline]
    pub unsafe fn get_dist_symbol(dist: c_int) -> c_int {
        zopfli_get_dist_symbol_wrapper(dist)
    }
    
    #[inline]
    pub unsafe fn get_length_extra_bits(l: c_int) -> c_int {
        zopfli_get_length_extra_bits_wrapper(l)
    }
    
    #[inline]
    pub unsafe fn get_length_extra_bits_value(l: c_int) -> c_int {
        zopfli_get_length_extra_bits_value_wrapper(l)
    }
    
    #[inline]
    pub unsafe fn get_length_symbol(l: c_int) -> c_int {
        zopfli_get_length_symbol_wrapper(l)
    }
    
    #[inline]
    pub unsafe fn get_length_symbol_extra_bits(s: c_int) -> c_int {
        zopfli_get_length_symbol_extra_bits_wrapper(s)
    }
    
    #[inline]
    pub unsafe fn get_dist_symbol_extra_bits(s: c_int) -> c_int {
        zopfli_get_dist_symbol_extra_bits_wrapper(s)
    }
}

// Convenience wrappers for options functions
#[cfg(feature = "c-fallback")]
pub mod options {
    use super::*;
    
    #[inline]
    pub unsafe fn init_options(options: *mut ZopfliOptions) {
        ZopfliInitOptions(options)
    }
}

// Convenience wrappers for tree/huffman functions
#[cfg(feature = "c-fallback")]
pub mod tree {
    use super::*;
    
    #[inline]
    pub unsafe fn length_limited_code_lengths(
        frequencies: *const usize, 
        n: c_int, 
        maxbits: c_int, 
        bitlengths: *mut u32
    ) -> c_int {
        ZopfliLengthLimitedCodeLengths(frequencies, n, maxbits, bitlengths)
    }
    
    #[inline]
    pub unsafe fn calculate_bit_lengths(
        count: *const usize, 
        n: usize, 
        maxbits: c_int,
        bitlengths: *mut u32
    ) {
        ZopfliCalculateBitLengths(count, n, maxbits, bitlengths)
    }
    
    #[inline]
    pub unsafe fn lengths_to_symbols(
        lengths: *const u32, 
        n: usize, 
        maxbits: u32,
        symbols: *mut u32
    ) {
        ZopfliLengthsToSymbols(lengths, n, maxbits, symbols)
    }
    
    #[inline]
    pub unsafe fn calculate_entropy(
        count: *const usize, 
        n: usize, 
        bitlengths: *mut c_double
    ) {
        ZopfliCalculateEntropy(count, n, bitlengths)
    }
}

// Convenience wrappers for hash functions
#[cfg(feature = "c-fallback")]
pub mod hash {
    use super::*;
    
    #[inline]
    pub unsafe fn alloc_hash(window_size: usize, h: *mut ZopfliHashC) {
        ZopfliAllocHash(window_size, h)
    }
    
    #[inline]
    pub unsafe fn reset_hash(window_size: usize, h: *mut ZopfliHashC) {
        ZopfliResetHash(window_size, h)
    }
    
    #[inline]
    pub unsafe fn clean_hash(h: *mut ZopfliHashC) {
        ZopfliCleanHash(h)
    }
    
    #[inline]
    pub unsafe fn update_hash(array: *const u8, pos: usize, end: usize, h: *mut ZopfliHashC) {
        ZopfliUpdateHash(array, pos, end, h)
    }
    
    #[inline]
    pub unsafe fn warmup_hash(array: *const u8, pos: usize, end: usize, h: *mut ZopfliHashC) {
        ZopfliWarmupHash(array, pos, end, h)
    }
}

// Convenience wrappers for cache functions
#[cfg(feature = "c-fallback")]
pub mod cache {
    use super::*;
    
    #[inline]
    pub unsafe fn init_cache(blocksize: usize, lmc: *mut ZopfliLongestMatchCacheC) {
        ZopfliInitCache(blocksize, lmc)
    }
    
    #[inline]
    pub unsafe fn clean_cache(lmc: *mut ZopfliLongestMatchCacheC) {
        ZopfliCleanCache(lmc)
    }
    
    #[inline]
    pub unsafe fn sublen_to_cache(sublen: *const u16, pos: usize, length: usize, lmc: *mut ZopfliLongestMatchCacheC) {
        ZopfliSublenToCache(sublen, pos, length, lmc)
    }
    
    #[inline]
    pub unsafe fn cache_to_sublen(lmc: *const ZopfliLongestMatchCacheC, pos: usize, length: usize, sublen: *mut u16) {
        ZopfliCacheToSublen(lmc, pos, length, sublen)
    }
    
    #[inline]
    pub unsafe fn max_cached_sublen(lmc: *const ZopfliLongestMatchCacheC, pos: usize, length: usize) -> u32 {
        ZopfliMaxCachedSublen(lmc, pos, length)
    }
}

