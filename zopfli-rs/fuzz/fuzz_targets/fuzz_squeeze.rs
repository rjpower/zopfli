#![no_main]
use libfuzzer_sys::fuzz_target;
use zopfli::{lz77::{ZopfliBlockState, ZopfliLZ77Store}, options::ZopfliOptions};
use std::os::raw::c_int;

// FFI for C implementations
extern "C" {
    fn ZopfliLZ77Optimal(
        s: *mut zopfli::ffi::ZopfliBlockStateC,
        input: *const u8,
        instart: usize,
        inend: usize,
        numiterations: c_int,
        store: *mut zopfli::ffi::ZopfliLZ77StoreC,
    );

    fn ZopfliLZ77OptimalFixed(
        s: *mut zopfli::ffi::ZopfliBlockStateC,
        input: *const u8,
        instart: usize,
        inend: usize,
        store: *mut zopfli::ffi::ZopfliLZ77StoreC,
    );
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    // Extract parameters from fuzz input
    let numiterations = (data[0] % 15) as i32 + 1; // 1-15 iterations
    let block_splitting = data[1] & 1 != 0;
    let add_lmc = data[2] & 1 != 0;
    let test_data = &data[3..];

    if test_data.is_empty() || test_data.len() > 100_000 {
        return;
    }

    // Create options
    let options = ZopfliOptions {
        verbose: 0,
        verbose_more: 0,
        numiterations,
        blocksplitting: if block_splitting { 1 } else { 0 },
        blocksplittinglast: 0,
        blocksplittingmax: 15,
    };

    // Test ZopfliLZ77Optimal
    {
        // Rust implementation
        let mut rust_block_state = match ZopfliBlockState::new(&options, 0, test_data.len(), add_lmc) {
            Ok(state) => state,
            Err(_) => return,
        };
        let mut rust_store = ZopfliLZ77Store::new(test_data);
        zopfli::squeeze::lz77_optimal(
            &mut rust_block_state,
            test_data,
            0,
            test_data.len(),
            numiterations,
            &mut rust_store,
        );

        // C implementation
        let mut c_block_state: zopfli::ffi::ZopfliBlockStateC = unsafe { std::mem::zeroed() };
        unsafe {
            zopfli::ffi::ZopfliInitBlockState(
                &options as *const ZopfliOptions,
                0,
                test_data.len(),
                if add_lmc { 1 } else { 0 },
                &mut c_block_state,
            );
        }

        let mut c_store: zopfli::ffi::ZopfliLZ77StoreC = unsafe { std::mem::zeroed() };
        unsafe {
            zopfli::ffi::ZopfliInitLZ77Store(test_data.as_ptr(), &mut c_store);
            
            ZopfliLZ77Optimal(
                &mut c_block_state,
                test_data.as_ptr(),
                0,
                test_data.len(),
                numiterations,
                &mut c_store,
            );
        }

        // Compare results exactly - the algorithms should be deterministic
        let c_size = unsafe { zopfli::ffi::ZopfliLZ77StoreGetSize(&c_store) };
        assert_eq!(rust_store.size(), c_size, "Store sizes differ: Rust={}, C={}", rust_store.size(), c_size);
        
        // Compare each LZ77 symbol
        for i in 0..rust_store.size() {
            let (rust_litlen, rust_dist) = rust_store.get_litlen_dist(i);
            let c_litlen = unsafe { zopfli::ffi::ZopfliLZ77StoreGetLitLen(&c_store, i) };
            let c_dist = unsafe { zopfli::ffi::ZopfliLZ77StoreGetDist(&c_store, i) };
            
            assert_eq!(rust_litlen, c_litlen, 
                "LitLen mismatch at index {}: Rust={}, C={}", i, rust_litlen, c_litlen);
            assert_eq!(rust_dist, c_dist,
                "Dist mismatch at index {}: Rust={}, C={}", i, rust_dist, c_dist);
        }
        
        // Clean up C resources
        unsafe {
            zopfli::ffi::ZopfliCleanLZ77Store(&mut c_store);
            zopfli::ffi::ZopfliCleanBlockState(&mut c_block_state);
        }
    }

    // Test ZopfliLZ77OptimalFixed
    {
        // Rust implementation
        let mut rust_block_state = match ZopfliBlockState::new(&options, 0, test_data.len(), add_lmc) {
            Ok(state) => state,
            Err(_) => return,
        };
        let mut rust_store = ZopfliLZ77Store::new(test_data);
        zopfli::squeeze::lz77_optimal_fixed(
            &mut rust_block_state,
            test_data,
            0,
            test_data.len(),
            &mut rust_store,
        );

        // C implementation
        let mut c_block_state: zopfli::ffi::ZopfliBlockStateC = unsafe { std::mem::zeroed() };
        unsafe {
            zopfli::ffi::ZopfliInitBlockState(
                &options as *const ZopfliOptions,
                0,
                test_data.len(),
                if add_lmc { 1 } else { 0 },
                &mut c_block_state,
            );
        }

        let mut c_store: zopfli::ffi::ZopfliLZ77StoreC = unsafe { std::mem::zeroed() };
        unsafe {
            zopfli::ffi::ZopfliInitLZ77Store(test_data.as_ptr(), &mut c_store);
            
            ZopfliLZ77OptimalFixed(
                &mut c_block_state,
                test_data.as_ptr(),
                0,
                test_data.len(),
                &mut c_store,
            );
        }

        // Compare results exactly for fixed tree
        let c_size = unsafe { zopfli::ffi::ZopfliLZ77StoreGetSize(&c_store) };
        assert_eq!(rust_store.size(), c_size, "Fixed store sizes differ: Rust={}, C={}", rust_store.size(), c_size);
        
        // Compare each LZ77 symbol
        for i in 0..rust_store.size() {
            let (rust_litlen, rust_dist) = rust_store.get_litlen_dist(i);
            let c_litlen = unsafe { zopfli::ffi::ZopfliLZ77StoreGetLitLen(&c_store, i) };
            let c_dist = unsafe { zopfli::ffi::ZopfliLZ77StoreGetDist(&c_store, i) };
            
            assert_eq!(rust_litlen, c_litlen, 
                "Fixed LitLen mismatch at index {}: Rust={}, C={}", i, rust_litlen, c_litlen);
            assert_eq!(rust_dist, c_dist,
                "Fixed Dist mismatch at index {}: Rust={}, C={}", i, rust_dist, c_dist);
        }
        
        // Clean up C resources
        unsafe {
            zopfli::ffi::ZopfliCleanLZ77Store(&mut c_store);
            zopfli::ffi::ZopfliCleanBlockState(&mut c_block_state);
        }
    }
});