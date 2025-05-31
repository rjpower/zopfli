use std::os::raw::{c_int, c_double};
use crate::options::ZopfliOptions;

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