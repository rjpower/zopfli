/// Get distance extra bits
pub fn get_dist_extra_bits(dist: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_dist_extra_bits(dist)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_dist_extra_bits(dist)
    }
}

/// Get distance extra bits value
pub fn get_dist_extra_bits_value(dist: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_dist_extra_bits_value(dist)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_dist_extra_bits_value(dist)
    }
}

/// Get distance symbol
pub fn get_dist_symbol(dist: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_dist_symbol(dist)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_dist_symbol(dist)
    }
}

/// Get length extra bits
pub fn get_length_extra_bits(l: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_length_extra_bits(l)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_length_extra_bits(l)
    }
}

/// Get length extra bits value
pub fn get_length_extra_bits_value(l: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_length_extra_bits_value(l)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_length_extra_bits_value(l)
    }
}

/// Get length symbol
pub fn get_length_symbol(l: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_length_symbol(l)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_length_symbol(l)
    }
}

/// Get length symbol extra bits
pub fn get_length_symbol_extra_bits(s: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_length_symbol_extra_bits(s)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_length_symbol_extra_bits(s)
    }
}

/// Get distance symbol extra bits
pub fn get_dist_symbol_extra_bits(s: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_dist_symbol_extra_bits(s)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_dist_symbol_extra_bits(s)
    }
}

/// Initialize ZopfliOptions with default values
/// This bridges between C ZopfliInitOptions and Rust Default::default()
pub fn init_options() -> crate::options::ZopfliOptions {
    #[cfg(feature = "c-fallback")]
    {
        let mut options = crate::options::ZopfliOptions {
            verbose: 999,          // Initialize with garbage values to test C initialization
            verbose_more: 888,
            numiterations: 777,
            blocksplitting: 666,
            blocksplittinglast: 555,
            blocksplittingmax: 444,
        };
        unsafe {
            crate::ffi::options::init_options(&mut options as *mut crate::options::ZopfliOptions);
        }
        options
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::options::ZopfliOptions::default()
    }
}

/// Calculate length-limited code lengths using Katajainen algorithm
pub fn length_limited_code_lengths(
    frequencies: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    #[cfg(feature = "c-fallback")]
    unsafe {
        let result = crate::ffi::tree::length_limited_code_lengths(
            frequencies.as_ptr(),
            frequencies.len() as i32,
            maxbits,
            bitlengths.as_mut_ptr(),
        );
        if result == 0 { Ok(()) } else { Err(()) }
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::tree::length_limited_code_lengths(frequencies, maxbits, bitlengths)
    }
}

/// Calculate bit lengths for Huffman tree
pub fn calculate_bit_lengths(
    count: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::tree::calculate_bit_lengths(
            count.as_ptr(),
            count.len(),
            maxbits,
            bitlengths.as_mut_ptr(),
        );
        Ok(())
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::tree::calculate_bit_lengths(count, maxbits, bitlengths)
    }
}

/// Convert Huffman code lengths to symbols
pub fn lengths_to_symbols(
    lengths: &[u32],
    maxbits: u32,
    symbols: &mut [u32],
) {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::tree::lengths_to_symbols(
            lengths.as_ptr(),
            lengths.len(),
            maxbits,
            symbols.as_mut_ptr(),
        );
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::tree::lengths_to_symbols(lengths, maxbits, symbols)
    }
}

/// Calculate entropy for each symbol
pub fn calculate_entropy(count: &[usize], bitlengths: &mut [f64]) {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::tree::calculate_entropy(
            count.as_ptr(),
            count.len(),
            bitlengths.as_mut_ptr(),
        );
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::tree::calculate_entropy(count, bitlengths)
    }
}