#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip very short inputs
    if data.len() < 10 {
        return;
    }
    
    // Create hash instances - RUST vs C comparison
    let window_size = 32768; // Use standard ZOPFLI_WINDOW_SIZE
    let mut rust_hash = zopfli::hash::ZopfliHash::new(window_size);
    
    // Create C hash properly
    let mut c_hash = Box::new(unsafe { std::mem::zeroed::<zopfli::ffi::ZopfliHashC>() });
    unsafe {
        zopfli::ffi::hash::alloc_hash(window_size, c_hash.as_mut());
        zopfli::ffi::hash::reset_hash(window_size, c_hash.as_mut());
    }
    
    // Test warmup
    let pos = 0;
    let end = data.len().min(100); // Limit to reasonable size for fuzzing
    
    rust_hash.warmup(data, pos, end);
    
    unsafe {
        zopfli::ffi::hash::warmup_hash(data.as_ptr(), pos, end, c_hash.as_mut());
    }
    
    // Test update operations - compare RUST vs C
    for i in 0..end.min(50) { // Limit iterations for performance
        rust_hash.update(data, i, end);
        
        unsafe {
            zopfli::ffi::hash::update_hash(data.as_ptr(), i, end, c_hash.as_mut());
        }
        
        // Compare hash values - Rust vs C
        let rust_val = rust_hash.val();
        let c_val = unsafe { (*c_hash).val };
        assert_eq!(rust_val, c_val, "Hash values differ at position {}: Rust={}, C={}", i, rust_val, c_val);
    }
    
    unsafe {
        zopfli::ffi::hash::clean_hash(c_hash.as_mut());
    }
});