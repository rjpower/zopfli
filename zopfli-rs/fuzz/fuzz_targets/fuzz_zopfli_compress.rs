#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (u8, Vec<u8>)| {
    let (format_u8, input) = data;
    
    // Limit input size to prevent timeout in fuzzing
    if input.len() > 1024 {
        return;
    }
    
    // Map format byte to valid ZopfliFormat
    let format = match format_u8 % 3 {
        0 => zopfli::zopfli::ZopfliFormat::Gzip,
        1 => zopfli::zopfli::ZopfliFormat::Zlib,
        2 => zopfli::zopfli::ZopfliFormat::Deflate,
        _ => unreachable!(),
    };
    
    let options = zopfli::options::ZopfliOptions::default();
    
    #[cfg(feature = "c-fallback")]
    {
        use std::ptr;
        use std::os::raw::c_uchar;
        
        // Test Rust implementation
        let mut rust_output = Vec::new();
        zopfli::zopfli::compress(&options, format, &input, &mut rust_output);
        
        // Test C implementation
        let mut c_output_ptr: *mut c_uchar = ptr::null_mut();
        let mut c_output_size = 0usize;
        
        unsafe {
            zopfli::ffi::ZopfliCompress(
                &options as *const _,
                format as i32,
                input.as_ptr(),
                input.len(),
                &mut c_output_ptr,
                &mut c_output_size
            );
            
            let c_output = std::slice::from_raw_parts(c_output_ptr, c_output_size).to_vec();
            libc::free(c_output_ptr as *mut libc::c_void);
            
            // Compare outputs
            assert_eq!(rust_output, c_output, 
                "Compression mismatch for format {:?} and input: {:?}", 
                format, input);
        }
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        // Test just that the Rust implementation doesn't panic
        let mut output = Vec::new();
        zopfli::zopfli::compress(&options, format, &input, &mut output);
        
        // Basic sanity checks based on format
        if !input.is_empty() {
            match format {
                zopfli::zopfli::ZopfliFormat::Gzip => {
                    assert!(output.len() >= 18, "Gzip output too short");
                    assert_eq!(output[0], 31, "Invalid gzip magic number 1");
                    assert_eq!(output[1], 139, "Invalid gzip magic number 2");
                }
                zopfli::zopfli::ZopfliFormat::Zlib => {
                    assert!(output.len() >= 6, "Zlib output too short");
                    // Check zlib header checksum
                    let cmf = output[0] as u32;
                    let flg = output[1] as u32;
                    assert_eq!((cmf * 256 + flg) % 31, 0, "Invalid zlib header checksum");
                }
                zopfli::zopfli::ZopfliFormat::Deflate => {
                    assert!(!output.is_empty(), "Deflate output should not be empty");
                }
            }
        }
    }
});