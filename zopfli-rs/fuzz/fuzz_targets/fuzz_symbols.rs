#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    dist: i32,
    length: i32,
    symbol: i32,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let input: FuzzInput = match u.arbitrary() {
        Ok(input) => input,
        Err(_) => return, // Skip invalid inputs
    };
    
    // Test distance-related functions
    test_distance_functions(input.dist);
    
    // Test length-related functions
    test_length_functions(input.length);
    
    // Test symbol-related functions
    test_symbol_functions(input.symbol);
});

fn test_distance_functions(dist: i32) {
    // Skip i32::MIN to avoid overflow in abs()
    if dist == i32::MIN {
        return;
    }
    
    // Ensure dist is positive to match C behavior
    let dist = dist.abs();
    
    // Test get_dist_extra_bits
    let c_bits = unsafe { zopfli::ffi::symbols::get_dist_extra_bits(dist) };
    let rust_bits = zopfli::symbols::get_dist_extra_bits(dist);
    
    assert_eq!(c_bits, rust_bits, 
              "Extra bits mismatch for dist {}: C returned {}, Rust returned {}", 
              dist, c_bits, rust_bits);
    
    let bridge_bits = zopfli::bridge::get_dist_extra_bits(dist);
    assert_eq!(c_bits, bridge_bits,
              "Extra bits bridge mismatch for dist {}: C returned {}, bridge returned {}",
              dist, c_bits, bridge_bits);
    
    // Test get_dist_extra_bits_value
    let c_value = unsafe { zopfli::ffi::symbols::get_dist_extra_bits_value(dist) };
    let rust_value = zopfli::symbols::get_dist_extra_bits_value(dist);
    
    assert_eq!(c_value, rust_value, 
              "Extra bits value mismatch for dist {}: C returned {}, Rust returned {}", 
              dist, c_value, rust_value);
    
    let bridge_value = zopfli::bridge::get_dist_extra_bits_value(dist);
    assert_eq!(c_value, bridge_value,
              "Extra bits value bridge mismatch for dist {}: C returned {}, bridge returned {}",
              dist, c_value, bridge_value);
              
    // Test get_dist_symbol
    let c_symbol = unsafe { zopfli::ffi::symbols::get_dist_symbol(dist) };
    let rust_symbol = zopfli::symbols::get_dist_symbol(dist);
    
    assert_eq!(c_symbol, rust_symbol, 
              "Dist symbol mismatch for dist {}: C returned {}, Rust returned {}", 
              dist, c_symbol, rust_symbol);
    
    let bridge_symbol = zopfli::bridge::get_dist_symbol(dist);
    assert_eq!(c_symbol, bridge_symbol,
              "Dist symbol bridge mismatch for dist {}: C returned {}, bridge returned {}",
              dist, c_symbol, bridge_symbol);
}

fn test_length_functions(length: i32) {
    // Handle overflow case for i32::MIN
    if length == i32::MIN {
        return;
    }
    
    // Test within valid DEFLATE length range (3-258)
    let length = 3 + (length.abs() % 256); // Range [3, 258]
    
    // Test get_length_extra_bits
    let c_bits = unsafe { zopfli::ffi::symbols::get_length_extra_bits(length) };
    let rust_bits = zopfli::symbols::get_length_extra_bits(length);
    
    assert_eq!(c_bits, rust_bits, 
              "Length extra bits mismatch for length {}: C returned {}, Rust returned {}", 
              length, c_bits, rust_bits);
    
    let bridge_bits = zopfli::bridge::get_length_extra_bits(length);
    assert_eq!(c_bits, bridge_bits,
              "Length extra bits bridge mismatch for length {}: C returned {}, bridge returned {}",
              length, c_bits, bridge_bits);
    
    // Test get_length_extra_bits_value
    let c_value = unsafe { zopfli::ffi::symbols::get_length_extra_bits_value(length) };
    let rust_value = zopfli::symbols::get_length_extra_bits_value(length);
    
    assert_eq!(c_value, rust_value, 
              "Length extra bits value mismatch for length {}: C returned {}, Rust returned {}", 
              length, c_value, rust_value);
    
    let bridge_value = zopfli::bridge::get_length_extra_bits_value(length);
    assert_eq!(c_value, bridge_value,
              "Length extra bits value bridge mismatch for length {}: C returned {}, bridge returned {}",
              length, c_value, bridge_value);
              
    // Test get_length_symbol
    let c_symbol = unsafe { zopfli::ffi::symbols::get_length_symbol(length) };
    let rust_symbol = zopfli::symbols::get_length_symbol(length);
    
    assert_eq!(c_symbol, rust_symbol, 
              "Length symbol mismatch for length {}: C returned {}, Rust returned {}", 
              length, c_symbol, rust_symbol);
    
    let bridge_symbol = zopfli::bridge::get_length_symbol(length);
    assert_eq!(c_symbol, bridge_symbol,
              "Length symbol bridge mismatch for length {}: C returned {}, bridge returned {}",
              length, c_symbol, bridge_symbol);
}

fn test_symbol_functions(symbol: i32) {
    // Handle overflow case for i32::MIN
    if symbol == i32::MIN {
        return;
    }
    
    // Test length symbol extra bits (symbols 257-285)
    let length_symbol = 257 + (symbol.abs() % 29); // Range [257, 285]
    
    let c_bits = unsafe { zopfli::ffi::symbols::get_length_symbol_extra_bits(length_symbol) };
    let rust_bits = zopfli::symbols::get_length_symbol_extra_bits(length_symbol);
    
    assert_eq!(c_bits, rust_bits, 
              "Length symbol extra bits mismatch for symbol {}: C returned {}, Rust returned {}", 
              length_symbol, c_bits, rust_bits);
    
    let bridge_bits = zopfli::bridge::get_length_symbol_extra_bits(length_symbol);
    assert_eq!(c_bits, bridge_bits,
              "Length symbol extra bits bridge mismatch for symbol {}: C returned {}, bridge returned {}",
              length_symbol, c_bits, bridge_bits);
    
    // Test distance symbol extra bits (symbols 0-29)
    let dist_symbol = symbol.abs() % 30; // Range [0, 29]
    
    let c_dist_bits = unsafe { zopfli::ffi::symbols::get_dist_symbol_extra_bits(dist_symbol) };
    let rust_dist_bits = zopfli::symbols::get_dist_symbol_extra_bits(dist_symbol);
    
    assert_eq!(c_dist_bits, rust_dist_bits, 
              "Dist symbol extra bits mismatch for symbol {}: C returned {}, Rust returned {}", 
              dist_symbol, c_dist_bits, rust_dist_bits);
    
    let bridge_dist_bits = zopfli::bridge::get_dist_symbol_extra_bits(dist_symbol);
    assert_eq!(c_dist_bits, bridge_dist_bits,
              "Dist symbol extra bits bridge mismatch for symbol {}: C returned {}, bridge returned {}",
              dist_symbol, c_dist_bits, bridge_dist_bits);
}