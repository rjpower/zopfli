# Rust Port Setup Guide

This document outlines the structure and setup for incrementally porting Zopfli
from C to Rust, with simultaneous evaluation capabilities and comprehensive fuzz
testing. 

## Current Project Structure

```
zopfli/
├── src/                    # Original C source
├── port/                   # Porting documentation
├── zopfli-rs/              # Rust implementation
│   ├── Cargo.toml
│   ├── build.rs            # Build script for C compilation
│   ├── src/
│   │   ├── lib.rs          # Main library entry
│   │   ├── ffi.rs          # FFI bindings to C functions
│   │   ├── bridge.rs       # C-Rust bridge for testing
│   │   └── ...             # Other modules as ported
│   ├── fuzz/
│   │   ├── Cargo.toml
│   │   └── fuzz_targets/
│   │       ├── fuzz_lz77.rs
│   │       ├── fuzz_deflate.rs
│   │       └── ...
```

## Hybrid Build System

### 1. Main Cargo.toml

```toml
[package]
name = "zopfli"
version = "0.1.0"
edition = "2021"

[features]
default = ["c-fallback"]
c-fallback = ["cc"]  # Use C implementation for unported functions
pure-rust = []       # Use only Rust (will panic on unported functions)

[dependencies]
libc = "0.2"

[build-dependencies]
cc = { version = "1.0", optional = true }

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"

[profile.dev]
# Enable debug symbols for better debugging
debug = true
# Enable overflow checks
overflow-checks = true

[profile.release-with-debug]
inherits = "release"
debug = true
```

### 2. Build Script (build.rs)

The build script compiles C code when needed:

```rust
#[cfg(feature = "c-fallback")]
fn main() {
    use cc::Build;
    
    println!("cargo:rerun-if-changed=../src/zopfli");
    
    Build::new()
        .files(glob::glob("../src/zopfli/*.c").unwrap()
            .filter_map(Result::ok)
            .filter(|p| !p.to_str().unwrap().contains("_bin.c")))
        .include("../src/zopfli")
        .opt_level(3)
        .flag_if_supported("-Wno-unused-parameter")
        .compile("zopfli_c");
}

#[cfg(not(feature = "c-fallback"))]
fn main() {}
```

## FFI Bridge Design

### Core Principle: Function-Level Replacement

Each C function can be individually replaced with a Rust implementation:

```rust
// ffi.rs - C function declarations
#[cfg(feature = "c-fallback")]
extern "C" {
    fn ZopfliInitOptions(options: *mut ZopfliOptions);
    fn ZopfliCompress(options: *const ZopfliOptions, 
                      output_type: c_int,
                      in_data: *const u8, 
                      insize: usize,
                      out: *mut *mut u8, 
                      outsize: *mut usize);
}

// bridge.rs - Routing layer
pub fn zopfli_compress(options: &ZopfliOptions, 
                      output_type: OutputType,
                      data: &[u8]) -> Vec<u8> {
    match cfg!(feature = "pure-rust") {
        true => compress_rust(options, output_type, data),
        false => compress_c_fallback(options, output_type, data),
    }
}

// Individual function routing
pub fn lz77_store_lit_len_dist(length: u16, dist: u16, pos: usize, store: &mut Lz77Store) {
    if cfg!(feature = "lz77-rust") {
        crate::lz77::store_lit_len_dist(length, dist, pos, store)
    } else {
        unsafe { ffi::ZopfliStoreLitLenDist(length, dist, pos, store.as_mut_ptr()) }
    }
}
```

## Fuzz Testing Infrastructure

### 1. Fuzz Setup (fuzz/Cargo.toml)

```toml
[package]
name = "zopfli-fuzz"
version = "0.0.0"
publish = false

[dependencies]
libfuzzer-sys = "0.4"
zopfli = { path = ".." }

[[bin]]
name = "fuzz_differential"
path = "fuzz_targets/fuzz_differential.rs"
test = false
doc = false
```

### 2. Differential Fuzzing Target

Compare C and Rust implementations:

```rust
// fuzz_targets/fuzz_differential.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() { return; }
    
    let options = ZopfliOptions::default();
    
    // Get output from C implementation
    let c_output = zopfli::with_c_implementation(|| {
        zopfli::compress(&options, OutputType::Gzip, data)
    });
    
    // Get output from Rust implementation
    let rust_output = zopfli::with_rust_implementation(|| {
        zopfli::compress(&options, OutputType::Gzip, data)
    });
    
    // Verify outputs match
    assert_eq!(c_output, rust_output, 
              "Mismatch for input len {}", data.len());
});
```

### 3. Function-Level Fuzzing

Test individual functions:

```rust
// fuzz_targets/fuzz_lz77.rs
fuzz_target!(|input: Lz77FuzzInput| {
    let mut c_store = Lz77Store::new();
    let mut rust_store = Lz77Store::new();
    
    // Apply same operations to both
    for op in &input.operations {
        match op {
            Op::StoreLitLenDist { length, dist, pos } => {
                ffi::ZopfliStoreLitLenDist(*length, *dist, *pos, &mut c_store);
                lz77::store_lit_len_dist(*length, *dist, *pos, &mut rust_store);
            }
        }
    }
    
    // Compare final states
    assert_eq!(c_store, rust_store);
});
```

## Verification Strategy

### 1. Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_matches_c() {
        let test_cases = vec![
            // ... test inputs
        ];
        
        for input in test_cases {
            let c_result = unsafe { ffi::c_function(&input) };
            let rust_result = rust_function(&input);
            assert_eq!(c_result, rust_result);
        }
    }
}
```

### 2. Integration Test Script

```python
#!/usr/bin/env python3
# scripts/compare_outputs.py

import subprocess
import sys
import hashlib

def compress_with_c(input_file):
    """Compress using C zopfli"""
    subprocess.run(["./zopfli", input_file], check=True)
    return hashlib.sha256(open(input_file + ".gz", "rb").read()).hexdigest()

def compress_with_rust(input_file):
    """Compress using Rust zopfli"""
    subprocess.run(["./target/release/zopfli-rs", input_file], check=True)
    return hashlib.sha256(open(input_file + ".gz", "rb").read()).hexdigest()

# Compare outputs...
```

## Porting Workflow

### 1. Select Function to Port
Choose a leaf function with minimal dependencies. Note that static inline functions in C headers require wrapper functions:
```bash
# Good candidates:
- ZopfliInitOptions
- ZopfliStoreLitLenDist  
- ZopfliGetDistSymbol (requires wrapper)
- ZopfliGetLengthSymbol (requires wrapper)
- ZopfliGetDistExtraBits (requires wrapper)
```

For static inline functions, create a C wrapper:
```c
// src/symbols_wrapper.c
#include "../../src/zopfli/symbols.h"

int zopfli_get_dist_extra_bits_wrapper(int dist) {
    return ZopfliGetDistExtraBits(dist);
}
```

### 2. Write Rust Implementation
1. Create module file (e.g., `src/symbols.rs`)
2. Implement function with same signature
3. Add feature flag for conditional compilation

### 3. Add Tests
1. Unit tests comparing with C
2. Fuzz target for the function
3. Integration test case

### 4. Verify
```bash
# Run tests
cargo test --all-features

# Run fuzzer (few minutes)
cargo +nightly fuzz run fuzz_differential -- -max_total_time=300

# Check benchmarks
cargo bench
```

### 5. Enable in Production
Add feature flag to default features once verified.

## Build Commands

```bash
# Build with C fallback (default)
cargo build --release

# Build pure Rust (will panic on unported functions)  
cargo build --release --no-default-features --features pure-rust

# Build for fuzzing
cd fuzz && cargo +nightly fuzz build

# Run specific fuzzer
cargo +nightly fuzz run fuzz_differential

# Run all tests including C comparison
cargo test --all-features

# Benchmark C vs Rust
cargo bench --features benchmark
```

## Safety Considerations

1. **No Direct Pointer Passing**: Use handle-based approach per RUST_PORTING.md
2. **Memory Layout**: Ensure Rust structs match C layout with `#[repr(C)]`
3. **Bounds Checking**: Rust implementations must validate all array accesses
4. **Integer Overflow**: Use checked arithmetic in debug, wrapping in release
5. **Lifetime Management**: Clear ownership model for allocated memory

## Performance Monitoring

Track performance regression with benchmarks:

```rust
// benches/compression.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_compress(c: &mut Criterion) {
    let data = include_bytes!("../testdata/alice.txt");
    
    c.bench_function("compress_c", |b| {
        b.iter(|| zopfli::with_c_implementation(|| {
            zopfli::compress(&Default::default(), OutputType::Gzip, data)
        }))
    });
    
    c.bench_function("compress_rust", |b| {
        b.iter(|| zopfli::with_rust_implementation(|| {
            zopfli::compress(&Default::default(), OutputType::Gzip, data)
        }))
    });
}
```

## CI Integration

```yaml
# .github/workflows/rust-port.yml
name: Rust Port CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Build C library
      run: make
    - name: Test Rust port
      run: |
        cd zopfli-rs
        cargo test --all-features
        cargo +nightly fuzz build
```

## Next Steps

1. Create the basic Rust project structure
2. Implement FFI bindings for all C functions  
3. Set up differential fuzzing
4. Port first simple function (e.g., `ZopfliInitOptions`)
5. Establish benchmark baseline
6. Begin systematic porting of modules