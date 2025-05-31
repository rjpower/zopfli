#![no_main]
use libfuzzer_sys::fuzz_target;
use std::ptr;

fuzz_target!(|data: &[u8]| {
    // Skip extremely small inputs that would cause issues
    if data.len() < 10 {
        return;
    }

    // Test block_split_simple - this is pure logic, no LZ77 dependency
    let blocksize = 1 + (data[0] as usize % 100); // 1-100
    let instart = data[1] as usize % data.len();
    let inend = (instart + 1 + (data[2] as usize % (data.len() - instart))).min(data.len());

    // Only test if we have a reasonable range
    if inend > instart {
        let c_result = unsafe {
            let mut splitpoints: *mut usize = ptr::null_mut();
            let mut npoints: usize = 0;

            zopfli::ffi::blocksplitter::block_split_simple(
                data.as_ptr(),
                instart,
                inend,
                blocksize,
                &mut splitpoints,
                &mut npoints,
            );

            let result = if npoints > 0 && !splitpoints.is_null() {
                std::slice::from_raw_parts(splitpoints, npoints).to_vec()
            } else {
                Vec::new()
            };

            if !splitpoints.is_null() {
                libc::free(splitpoints as *mut libc::c_void);
            }

            result
        };

        let rust_result =
            zopfli::blocksplitter::block_split_simple(data, instart, inend, blocksize);

        assert_eq!(
            c_result, rust_result,
            "block_split_simple mismatch for instart={}, inend={}, blocksize={}",
            instart, inend, blocksize
        );

        // Test with bridge functions
        let rust_result = zopfli::bridge::block_split_simple(data, instart, inend, blocksize);
        assert!(rust_result.iter().all(|&x| x >= instart && x < inend));

        // For larger inputs, test the more complex functions that require LZ77
        if data.len() >= 50 && inend - instart >= 20 {
            let options = zopfli::bridge::init_options();
            let maxblocks = 1 + (data[3] as usize % 10); // 1-10 blocks max

            // Skip testing block_split and block_split_lz77 for now since they require
            // full LZ77 processing and deflate cost calculation which is complex and slow.
            // These will be tested once deflate.rs is implemented properly.
            // For now, just test that they don't panic:

            #[cfg(not(feature = "c-fallback"))]
            {
                // Now that deflate is implemented, test actual block splitting
                let rust_result = zopfli::bridge::block_split(&options, data, instart, inend, maxblocks);
                assert!(rust_result.iter().all(|&x| x >= instart && x < inend));
                assert!(rust_result.len() <= maxblocks.saturating_sub(1));
            }
        }
    }
});
