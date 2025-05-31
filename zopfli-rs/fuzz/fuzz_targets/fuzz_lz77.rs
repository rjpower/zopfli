#![no_main]
use libfuzzer_sys::fuzz_target;
use std::os::raw::{c_int, c_ushort};
use std::mem::MaybeUninit;

fuzz_target!(|data: &[u8]| {
    // Don't test on empty data
    if data.len() == 0 {
        return;
    }
    
    // Test various positions, but not too many to keep fuzzing fast
    let positions_to_test = std::cmp::min(data.len(), 20);
    
    for i in 0..positions_to_test {
        let pos = i * data.len() / positions_to_test;
        if pos >= data.len() {
            continue;
        }
        
        // Test with different limits
        for &limit in &[zopfli::lz77::ZOPFLI_MIN_MATCH, 10, 50, zopfli::lz77::ZOPFLI_MAX_MATCH] {
            if limit > data.len() - pos {
                continue;
            }
            
            // Test both with and without cache
            for use_cache in [0, 1] {
                test_find_longest_match(data, pos, limit, use_cache == 1);
            }
        }
    }
});

fn test_find_longest_match(data: &[u8], pos: usize, limit: usize, use_cache: bool) {
    let mut c_distance = 0u16;
    let mut c_length = 0u16;
    let mut rust_distance = 0u16;
    let mut rust_length = 0u16;
    
    // Call C implementation using direct FFI
    unsafe {
        // Initialize options
        let mut options = zopfli::options::ZopfliOptions::default();
        zopfli::ffi::ZopfliInitOptions(&mut options);
        
        // Initialize block state
        let mut block_state = MaybeUninit::<zopfli::ffi::ZopfliBlockStateC>::uninit();
        zopfli::ffi::ZopfliInitBlockState(
            &options,
            0,
            data.len(),
            if use_cache { 1 } else { 0 },
            block_state.as_mut_ptr()
        );
        let mut block_state = block_state.assume_init();
        
        // Initialize hash
        let mut hash = MaybeUninit::<zopfli::ffi::ZopfliHashC>::uninit();
        zopfli::ffi::hash::alloc_hash(zopfli::lz77::ZOPFLI_WINDOW_SIZE, hash.as_mut_ptr());
        let mut hash = hash.assume_init();
        zopfli::ffi::hash::reset_hash(zopfli::lz77::ZOPFLI_WINDOW_SIZE, &mut hash);
        
        // Initialize hash up to the current position
        if data.len() > 0 {
            // Warmup the hash for the data up to current position
            let warmup_end = if pos > 0 { pos + 1 } else { data.len().min(3) };
            zopfli::ffi::hash::warmup_hash(data.as_ptr(), 0, warmup_end, &mut hash);
            
            // Update hash for each position up to current pos
            for i in 0..=pos {
                if i >= data.len() {
                    break;
                }
                zopfli::ffi::hash::update_hash(data.as_ptr(), i, data.len(), &mut hash);
            }
        }
        
        // Call the C function
        zopfli::ffi::ZopfliFindLongestMatch(
            &mut block_state,
            &hash,
            data.as_ptr(),
            pos,
            data.len(),
            limit,
            std::ptr::null_mut(), // No sublen for now
            &mut c_distance,
            &mut c_length
        );
        
        // Cleanup
        zopfli::ffi::hash::clean_hash(&mut hash);
        zopfli::ffi::ZopfliCleanBlockState(&mut block_state);
    }
    
    // Call Rust implementation
    let options = zopfli::options::ZopfliOptions::default();
    let mut state = zopfli::lz77::ZopfliBlockState::new(&options, 0, data.len(), use_cache).unwrap();
    let mut hash = zopfli::hash::ZopfliHash::new(zopfli::lz77::ZOPFLI_WINDOW_SIZE);
    
    // Initialize hash with the data - match the C implementation exactly
    hash.reset(zopfli::lz77::ZOPFLI_WINDOW_SIZE);
    if data.len() > 0 {
        // Warmup the hash for the data up to current position
        let warmup_end = if pos > 0 { pos + 1 } else { data.len().min(3) };
        hash.warmup(data, 0, warmup_end);
        
        // Update hash for each position up to current pos
        for i in 0..=pos {
            if i >= data.len() {
                break;
            }
            hash.update(data, i, data.len());
        }
    }
    
    zopfli::lz77::find_longest_match(
        &mut state,
        &hash,
        data,
        pos,
        data.len(),
        limit,
        None,
        &mut rust_distance,
        &mut rust_length,
    );
    
    // Compare results
    assert_eq!(c_distance, rust_distance, 
        "Distance mismatch at pos {} with data len {}: C={}, Rust={}", 
        pos, data.len(), c_distance, rust_distance);
    assert_eq!(c_length, rust_length, 
        "Length mismatch at pos {} with data len {}: C={}, Rust={}", 
        pos, data.len(), c_length, rust_length);
    
    // If a match was found, verify it's valid
    if rust_length >= 3 && rust_distance > 0 {
        zopfli::lz77::verify_len_dist(data, pos, rust_distance, rust_length);
    }
}