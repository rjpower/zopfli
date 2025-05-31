#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip very short inputs
    if data.len() < 4 {
        return;
    }

    // Use first byte to determine maxbits (1-15)
    let maxbits = ((data[0] % 15) + 1) as i32;

    // Use remaining bytes as frequencies
    let frequencies: Vec<usize> = data[1..].iter().map(|&b| b as usize).collect();

    if frequencies.is_empty() {
        return; // Skip empty or too large inputs
    }

    // Test length_limited_code_lengths
    let mut c_bitlengths = vec![0u32; frequencies.len()];
    let mut rust_bitlengths = vec![0u32; frequencies.len()];

    let c_result = unsafe {
        zopfli::ffi::tree::length_limited_code_lengths(
            frequencies.as_ptr(),
            frequencies.len() as i32,
            maxbits,
            c_bitlengths.as_mut_ptr(),
        )
    };

    let rust_result =
        zopfli::tree::length_limited_code_lengths(&frequencies, maxbits, &mut rust_bitlengths);

    assert_eq!(c_result == 0, rust_result.is_ok());
    if c_result == 0 {
        assert_eq!(c_bitlengths, rust_bitlengths);
    }

    // Test calculate_entropy (should never fail)
    let mut c_entropy = vec![0.0; frequencies.len()];
    let mut rust_entropy = vec![0.0; frequencies.len()];

    unsafe {
        zopfli::ffi::tree::calculate_entropy(
            frequencies.as_ptr(),
            frequencies.len(),
            c_entropy.as_mut_ptr(),
        );
    }

    zopfli::tree::calculate_entropy(&frequencies, &mut rust_entropy);

    for i in 0..frequencies.len() {
        assert!((c_entropy[i] - rust_entropy[i]).abs() < 1e-10);
    }
});
