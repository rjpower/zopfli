#![no_main]
use libfuzzer_sys::fuzz_target;
use std::os::raw::c_int;
use std::mem::MaybeUninit;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    
    // Use first byte to determine which function to test
    let test_selector = data[0] % 5;
    let test_data = &data[1..];
    
    match test_selector {
        0 => test_lz77_store(test_data),
        1 => test_get_byte_range(test_data),
        2 => test_get_histogram(test_data),
        3 => test_append_store(test_data),
        4 => test_find_longest_match_comprehensive(test_data),
        _ => unreachable!(),
    }
});

fn test_lz77_store(data: &[u8]) {
    // Test ZopfliLZ77Store operations
    let mut rust_store = zopfli::lz77::ZopfliLZ77Store::new(data);
    
    // Test storing literals and length-distance pairs
    for (i, &byte) in data.iter().enumerate() {
        if i % 3 == 0 {
            // Store as literal
            rust_store.store_lit_len_dist(byte as u16, 0, i);
        } else if i >= 3 {
            // Store as length-distance pair if we have enough data
            let length = 3 + (byte % 10) as u16;
            let dist = 1 + (byte as u16 * 7) % (i as u16);
            rust_store.store_lit_len_dist(length, dist, i);
        }
    }
    
    // Test size
    assert!(rust_store.size() > 0 || data.is_empty());
    
    // Test clone
    let cloned = rust_store.clone();
    assert_eq!(cloned.size(), rust_store.size());
}

fn test_get_byte_range(data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    
    let mut store = zopfli::lz77::ZopfliLZ77Store::new(data);
    
    // Add some data
    for (i, &byte) in data.iter().enumerate().take(10) {
        store.store_lit_len_dist(byte as u16, 0, i);
    }
    
    if store.size() > 1 {
        let range1 = store.get_byte_range(0, 1);
        let range2 = store.get_byte_range(0, store.size());
        assert!(range1 <= range2);
    }
}

fn test_get_histogram(data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    
    let mut store = zopfli::lz77::ZopfliLZ77Store::new(data);
    
    // Add mixed literals and length-distance pairs
    for (i, &byte) in data.iter().enumerate().take(50) {
        if byte % 2 == 0 {
            store.store_lit_len_dist(byte as u16, 0, i);
        } else if i >= 3 {
            let length = 3 + (byte % 20) as u16;
            let dist = 1 + (byte as u16) % 100;
            store.store_lit_len_dist(length, dist, i);
        }
    }
    
    if store.size() > 0 {
        let mut ll_counts = vec![0; 288];
        let mut d_counts = vec![0; 32];
        store.get_histogram(0, store.size(), &mut ll_counts, &mut d_counts);
        
        // Verify histogram has some counts
        let total_ll: usize = ll_counts.iter().sum();
        assert_eq!(total_ll, store.size());
    }
}

fn test_append_store(data: &[u8]) {
    if data.len() < 4 {
        return;
    }
    
    let mid = data.len() / 2;
    let data1 = &data[..mid];
    let data2 = &data[mid..];
    
    let mut store1 = zopfli::lz77::ZopfliLZ77Store::new(data);
    let mut store2 = zopfli::lz77::ZopfliLZ77Store::new(data);
    
    // Add data to both stores
    for (i, &byte) in data1.iter().enumerate() {
        store1.store_lit_len_dist(byte as u16, 0, i);
    }
    
    for (i, &byte) in data2.iter().enumerate() {
        store2.store_lit_len_dist(byte as u16, 0, i);
    }
    
    let size1 = store1.size();
    let size2 = store2.size();
    
    store1.append_store(&store2);
    assert_eq!(store1.size(), size1 + size2);
}

fn test_find_longest_match_comprehensive(data: &[u8]) {
    if data.len() == 0 || data.len() > 10000 {
        return;
    }
    
    // Test with different positions
    let positions_to_test = std::cmp::min(data.len(), 10);
    
    for i in 0..positions_to_test {
        let pos = i * data.len() / positions_to_test;
        if pos >= data.len() {
            continue;
        }
        
        // Test with different limits
        for &limit in &[zopfli::lz77::ZOPFLI_MIN_MATCH, 10, zopfli::lz77::ZOPFLI_MAX_MATCH] {
            if limit > data.len() - pos {
                continue;
            }
            
            // Compare C and Rust implementations
            compare_find_longest_match(data, pos, limit, false);
            compare_find_longest_match(data, pos, limit, true);
        }
    }
}

fn compare_find_longest_match(data: &[u8], pos: usize, limit: usize, use_cache: bool) {
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
            let warmup_end = if pos > 0 { pos + 1 } else { data.len().min(3) };
            zopfli::ffi::hash::warmup_hash(data.as_ptr(), 0, warmup_end, &mut hash);
            
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
            std::ptr::null_mut(),
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
    
    // Initialize hash - match the C implementation exactly
    hash.reset(zopfli::lz77::ZOPFLI_WINDOW_SIZE);
    if data.len() > 0 {
        let warmup_end = if pos > 0 { pos + 1 } else { data.len().min(3) };
        hash.warmup(data, 0, warmup_end);
        
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
}