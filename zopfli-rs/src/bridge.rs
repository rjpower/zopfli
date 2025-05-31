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

/// Bridge for ZopfliHash - provides unified interface for C and Rust implementations
pub struct ZopfliHashBridge {
    #[cfg(feature = "c-fallback")]
    c_hash: Box<crate::ffi::ZopfliHashC>,
    #[cfg(not(feature = "c-fallback"))]
    rust_hash: crate::hash::ZopfliHash,
}

impl ZopfliHashBridge {
    pub fn new(window_size: usize) -> Self {
        #[cfg(feature = "c-fallback")]
        {
            let mut c_hash = Box::new(unsafe { std::mem::zeroed::<crate::ffi::ZopfliHashC>() });
            unsafe {
                crate::ffi::hash::alloc_hash(window_size, c_hash.as_mut());
                crate::ffi::hash::reset_hash(window_size, c_hash.as_mut());
            }
            ZopfliHashBridge { c_hash }
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            ZopfliHashBridge {
                rust_hash: crate::hash::ZopfliHash::new(window_size),
            }
        }
    }
    
    pub fn update(&mut self, array: &[u8], pos: usize, end: usize) {
        #[cfg(feature = "c-fallback")]
        unsafe {
            crate::ffi::hash::update_hash(array.as_ptr(), pos, end, self.c_hash.as_mut());
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            self.rust_hash.update(array, pos, end);
        }
    }
    
    pub fn warmup(&mut self, array: &[u8], pos: usize, end: usize) {
        #[cfg(feature = "c-fallback")]
        unsafe {
            crate::ffi::hash::warmup_hash(array.as_ptr(), pos, end, self.c_hash.as_mut());
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            self.rust_hash.warmup(array, pos, end);
        }
    }
}

#[cfg(feature = "c-fallback")]
impl Drop for ZopfliHashBridge {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::hash::clean_hash(self.c_hash.as_mut());
        }
    }
}

/// Bridge for ZopfliLongestMatchCache - provides unified interface for C and Rust implementations
pub struct ZopfliLongestMatchCacheBridge {
    #[cfg(feature = "c-fallback")]
    c_cache: Box<crate::ffi::ZopfliLongestMatchCacheC>,
    #[cfg(not(feature = "c-fallback"))]
    rust_cache: crate::cache::ZopfliLongestMatchCache,
}

impl ZopfliLongestMatchCacheBridge {
    pub fn new(blocksize: usize) -> Result<Self, String> {
        #[cfg(feature = "c-fallback")]
        {
            let mut c_cache = Box::new(unsafe { std::mem::zeroed::<crate::ffi::ZopfliLongestMatchCacheC>() });
            unsafe {
                crate::ffi::cache::init_cache(blocksize, c_cache.as_mut());
            }
            Ok(ZopfliLongestMatchCacheBridge { c_cache })
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            let rust_cache = crate::cache::ZopfliLongestMatchCache::new(blocksize)?;
            Ok(ZopfliLongestMatchCacheBridge { rust_cache })
        }
    }
    
    pub fn sublen_to_cache(&mut self, sublen: &[u16], pos: usize, length: usize) {
        #[cfg(feature = "c-fallback")]
        unsafe {
            crate::ffi::cache::sublen_to_cache(sublen.as_ptr(), pos, length, self.c_cache.as_mut());
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            self.rust_cache.sublen_to_cache(sublen, pos, length);
        }
    }
    
    pub fn cache_to_sublen(&self, pos: usize, length: usize, sublen: &mut [u16]) {
        #[cfg(feature = "c-fallback")]
        unsafe {
            crate::ffi::cache::cache_to_sublen(self.c_cache.as_ref(), pos, length, sublen.as_mut_ptr());
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            self.rust_cache.cache_to_sublen(pos, length, sublen);
        }
    }
    
    pub fn max_cached_sublen(&self, pos: usize, length: usize) -> usize {
        #[cfg(feature = "c-fallback")]
        unsafe {
            crate::ffi::cache::max_cached_sublen(self.c_cache.as_ref(), pos, length) as usize
        }
        
        #[cfg(not(feature = "c-fallback"))]
        {
            self.rust_cache.max_cached_sublen(pos, length)
        }
    }
}

#[cfg(feature = "c-fallback")]
impl Drop for ZopfliLongestMatchCacheBridge {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::cache::clean_cache(self.c_cache.as_mut());
        }
    }
}