#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse fuzz input
    if data.len() < 16 {
        return;
    }
    
    let blocksize = ((data[0] as usize) % 1000) + 10; // 10-1009
    let pos = (data[1] as usize) % blocksize;
    let length = ((data[2] as usize) % 100) + 3; // 3-102
    
    // Create sublen array from remaining data
    let mut sublen = vec![0u16; length + 1];
    for i in 3..length.min(data.len() - 3) {
        if i + 3 < data.len() {
            sublen[i] = ((data[i + 3] as u16) << 8) | (data[(i + 4) % data.len()] as u16);
        }
    }
    
    // Test Rust implementation
    let mut rust_cache = match zopfli::cache::ZopfliLongestMatchCache::new(blocksize) {
        Ok(cache) => cache,
        Err(_) => return, // Skip if allocation fails
    };
    
    #[cfg(feature = "c-fallback")]
    let mut c_cache = {
        let mut c_cache = Box::new(unsafe { std::mem::zeroed::<zopfli::ffi::ZopfliLongestMatchCacheC>() });
        unsafe {
            zopfli::ffi::cache::init_cache(blocksize, c_cache.as_mut());
        }
        c_cache
    };
    
    // Test sublen_to_cache
    rust_cache.sublen_to_cache(&sublen, pos, length);
    
    #[cfg(feature = "c-fallback")]
    unsafe {
        zopfli::ffi::cache::sublen_to_cache(sublen.as_ptr(), pos, length, c_cache.as_mut());
    }
    
    // Test max_cached_sublen
    let rust_max = rust_cache.max_cached_sublen(pos, length);
    
    #[cfg(feature = "c-fallback")]
    let c_max = unsafe {
        zopfli::ffi::cache::max_cached_sublen(c_cache.as_ref(), pos, length) as usize
    };
    
    #[cfg(feature = "c-fallback")]
    assert_eq!(rust_max, c_max, "max_cached_sublen differs: rust={}, c={}", rust_max, c_max);
    
    // Test cache_to_sublen if we have valid cached data
    if rust_max > 0 {
        let mut rust_reconstructed = vec![0u16; rust_max + 1];
        let mut c_reconstructed = vec![0u16; rust_max + 1];
        
        rust_cache.cache_to_sublen(pos, length, &mut rust_reconstructed);
        
        #[cfg(feature = "c-fallback")]
        unsafe {
            zopfli::ffi::cache::cache_to_sublen(c_cache.as_ref(), pos, length, c_reconstructed.as_mut_ptr());
        }
        
        #[cfg(feature = "c-fallback")]
        {
            // Compare reconstructed data up to the cached length
            for i in 3..=rust_max.min(rust_reconstructed.len() - 1) {
                assert_eq!(rust_reconstructed[i], c_reconstructed[i], 
                          "Reconstructed sublen differs at index {}: rust={}, c={}", i, rust_reconstructed[i], c_reconstructed[i]);
            }
        }
    }
    
    #[cfg(feature = "c-fallback")]
    unsafe {
        zopfli::ffi::cache::clean_cache(c_cache.as_mut());
    }
});