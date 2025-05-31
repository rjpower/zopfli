#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent timeout in fuzzing
    if data.len() > 1024 {
        return;
    }
    
    let options = zopfli::options::ZopfliOptions::default();
    
    #[cfg(feature = "c-fallback")]
    {
        use std::ptr;
        use std::os::raw::c_uchar;
        
        // Test Rust implementation
        let mut rust_output = Vec::new();
        zopfli::gzip_container::gzip_compress(&options, data, &mut rust_output);
        
        // Test C implementation
        let mut c_output_ptr: *mut c_uchar = ptr::null_mut();
        let mut c_output_size = 0usize;
        
        unsafe {
            zopfli::ffi::ZopfliGzipCompress(
                &options as *const _,
                data.as_ptr(),
                data.len(),
                &mut c_output_ptr,
                &mut c_output_size
            );
            
            let c_output = std::slice::from_raw_parts(c_output_ptr, c_output_size).to_vec();
            libc::free(c_output_ptr as *mut libc::c_void);
            
            // Compare outputs
            assert_eq!(rust_output, c_output, 
                "Gzip compression mismatch for input: {:?}", 
                data);
        }
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        // Test just that the Rust implementation doesn't panic
        let mut output = Vec::new();
        zopfli::gzip_container::gzip_compress(&options, data, &mut output);
        
        // Basic sanity checks for gzip format
        if !data.is_empty() {
            assert!(output.len() >= 18, "Gzip output too short: {} bytes", output.len());
            assert_eq!(output[0], 31, "Invalid gzip magic number 1");
            assert_eq!(output[1], 139, "Invalid gzip magic number 2");
            assert_eq!(output[2], 8, "Invalid compression method");
        }
    }
});