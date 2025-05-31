#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip extremely small inputs that would cause issues
    if data.len() < 10 {
        return;
    }

    // Create LZ77 representation
    let mut rust_store = zopfli::lz77::ZopfliLZ77Store::new(data);
    let options = zopfli::options::ZopfliOptions::default();
    let mut s = zopfli::lz77::ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
    let mut hash = zopfli::hash::ZopfliHash::new(zopfli::util::ZOPFLI_WINDOW_SIZE);
    
    // Create LZ77 representation
    zopfli::lz77::lz77_greedy(&mut s, data, 0, data.len(), &mut rust_store, &mut hash);
    
    if rust_store.size() == 0 {
        return; // Skip empty stores
    }

    // Test block size calculations for different types
    let lstart = 0;
    let lend = rust_store.size();
    
    // Compare with C implementation
    let litlens: Vec<u16> = rust_store.litlens().to_vec();
    let dists: Vec<u16> = rust_store.dists().to_vec();
    let pos: Vec<usize> = rust_store.pos().to_vec();
    let ll_symbol: Vec<u16> = rust_store.ll_symbol().to_vec();
    let d_symbol: Vec<u16> = rust_store.d_symbol().to_vec();
    let ll_counts: Vec<usize> = rust_store.ll_counts().to_vec();
    let d_counts: Vec<usize> = rust_store.d_counts().to_vec();
    
    let c_store = zopfli::ffi::ZopfliLZ77StoreC {
        litlens: litlens.as_ptr() as *mut u16,
        dists: dists.as_ptr() as *mut u16,
        size: rust_store.size(),
        data: rust_store.data().as_ptr(),
        pos: pos.as_ptr() as *mut usize,
        ll_symbol: ll_symbol.as_ptr() as *mut u16,
        d_symbol: d_symbol.as_ptr() as *mut u16,
        ll_counts: ll_counts.as_ptr() as *mut usize,
        d_counts: d_counts.as_ptr() as *mut usize,
    };
    
    #[cfg(feature = "c-fallback")]
    unsafe {
        // Test each block type
        for btype in 0..=2 {
            let c_size = zopfli::ffi::ZopfliCalculateBlockSize(&c_store as *const _, lstart, lend, btype);
            let rust_size = zopfli::deflate::calculate_block_size(&rust_store, lstart, lend, btype);
            
            // Allow small differences due to floating point precision
            let tolerance = 1.0;
            assert!(
                (c_size - rust_size).abs() < tolerance,
                "Block size mismatch for type {}: C={:.2}, Rust={:.2}",
                btype, c_size, rust_size
            );
        }
        
        // Test auto type selection
        let c_auto = zopfli::ffi::ZopfliCalculateBlockSizeAutoType(&c_store as *const _, lstart, lend);
        let rust_auto = zopfli::deflate::calculate_block_size_auto_type(&rust_store, lstart, lend);
        
        let tolerance = 1.0;
        assert!(
            (c_auto - rust_auto).abs() < tolerance,
            "Auto type block size mismatch: C={:.2}, Rust={:.2}",
            c_auto, rust_auto
        );
    }
    
    // Test that functions work and return reasonable values
    let uncompressed = zopfli::deflate::calculate_block_size(&rust_store, lstart, lend, 0);
    let fixed = zopfli::deflate::calculate_block_size(&rust_store, lstart, lend, 1);
    let dynamic = zopfli::deflate::calculate_block_size(&rust_store, lstart, lend, 2);
    let auto = zopfli::deflate::calculate_block_size_auto_type(&rust_store, lstart, lend);
    
    // Basic sanity checks
    assert!(uncompressed > 0.0, "Uncompressed size should be positive");
    assert!(fixed > 0.0, "Fixed size should be positive");
    assert!(dynamic > 0.0, "Dynamic size should be positive");
    assert!(auto > 0.0, "Auto size should be positive");
    
    // Auto should pick the minimum
    let min_size = uncompressed.min(fixed).min(dynamic);
    assert!(
        (auto - min_size).abs() < 1e-10,
        "Auto should pick minimum: auto={:.2}, min={:.2}",
        auto, min_size
    );
    
    // Test block splitting works
    let splits = zopfli::blocksplitter::block_split(&options, data, 0, data.len(), 5);
    assert!(splits.iter().all(|&x| x < data.len()));
});