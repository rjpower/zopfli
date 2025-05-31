# Single Function/Struct Porting Guide for Zopfli

This guide provides practical instructions for porting a single function or
struct from the Zopfli C codebase to Rust.

You will be given one or more tasks from @port/TASKS.md.
MARK TASKS AS COMPLETED WHEN YOU HAVE COMPLETED THE PORTING.

## Project Structure Overview

```
zopfli/
├── src/zopfli/          # Original C source
├── port/                # Porting documentation
├── zopfli-rs/           # Rust implementation
│   ├── src/
│   │   ├── lib.rs       # Minimal module declarations
│   │   ├── ffi.rs       # C function declarations and wrappers
│   │   ├── bridge.rs    # Routing between C and Rust implementations
│   │   ├── symbols.rs   # Example: Rust implementations of symbol functions
│   │   └── *.rs         # Other modules as you port them
│   ├── fuzz/
│   │   └── fuzz_targets/
│   │       └── fuzz_*.rs # One fuzz target per function
│   └── tests/
│       └── test_*.rs    # Exhaustive comparison tests
```

## Key Principles

1. **Minimize scope**: Only implement what's needed for the specific function/struct being ported
2. **Test exhaustively**: Use both comprehensive unit tests and fuzzing
3. **Handle C quirks**: inline functions need wrappers, watch for integer overflow edge cases
4. **Keep it simple**: Don't build infrastructure you don't need yet


## Step-by-Step Porting Process

### 1. Analyze the C Function/Struct

First, understand what you're porting:
- Check if it's a static inline function (requires wrapper)
- Identify dependencies (other functions/structs it uses)
- Note any macros or constants used
- Check for platform-specific code or undefined behavior

Example findings for `ZopfliGetDistExtraBits`:
- Static inline function in `symbols.h`
- Uses `__builtin_clz` (maps to `leading_zeros()` in Rust)
- Takes `int`, returns `int`
- No dependencies on other functions

### Handle Static Inline Functions

If the C function is `inline`, it won't be exported from C, so we can't compare against it.
Create a wrapper to export the function. See `src/symbols_wrapper.c` for an example.

```c
// src/symbols_wrapper.c
#include "../../src/zopfli/symbols.h"

// Wrapper for static inline function
int zopfli_get_dist_extra_bits_wrapper(int dist) {
    return ZopfliGetDistExtraBits(dist);
}
```

Update `build.rs` to compile only what you need:
```rust
// build.rs
#[cfg(feature = "c-fallback")]
fn main() {
    use cc::Build;
    
    println!("cargo:rerun-if-changed=src/symbols_wrapper.c");
    println!("cargo:rerun-if-changed=../../src/zopfli/symbols.h");
    
    Build::new()
        .file("src/symbols_wrapper.c")
        .include("../../src/zopfli")
        .compile("symbols_wrapper");
}
```

### 2. Create FFI Bindings

Keep FFI minimal - only declare what you need:

```rust
// src/ffi.rs
use std::os::raw::c_int;

#[cfg(feature = "c-fallback")]
extern "C" {
    pub fn zopfli_get_dist_extra_bits_wrapper(dist: c_int) -> c_int;
}

#[cfg(feature = "c-fallback")]
pub mod symbols {
    use super::*;
    
    #[inline]
    pub unsafe fn get_dist_extra_bits(dist: c_int) -> c_int {
        zopfli_get_dist_extra_bits_wrapper(dist)
    }
}
```

### 3. Implement the Rust Version

Create a focused implementation, with a goal of being as close to the C implementation as possible.
Follow the guidelines from @port/RUST_PORTING.md for how to write good Rust code.

```rust
// src/symbols.rs
/// Gets the amount of extra bits for the given dist, cfr. the DEFLATE spec.
pub fn get_dist_extra_bits(dist: i32) -> i32 {
    if dist < 5 {
        return 0;
    }
    
    // Using leading_zeros() which is equivalent to __builtin_clz
    let l = 31 - (dist - 1).leading_zeros() as i32;
    l - 1 // log2(dist - 1) - 1
}
```

Key mappings:
- `__builtin_clz(x)` → `x.leading_zeros()`
- Be careful with integer types and overflow

### 6. Create the Bridge

If implementing a function, add your function to the bridge.

The bridge routes between C and Rust based on features:

```rust
// src/bridge.rs
/// Get distance extra bits
pub fn get_dist_extra_bits(dist: i32) -> i32 {
    #[cfg(feature = "c-fallback")]
    unsafe {
        crate::ffi::symbols::get_dist_extra_bits(dist)
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        crate::symbols::get_dist_extra_bits(dist)
    }
}
```

### 7. Link into lib

```rust
// src/lib.rs - Just module declarations
pub mod ffi;
pub mod bridge;
pub mod symbols;

/// your new module if appropriate
```

### 7. Write Tests

Create an inline test for your function testing basic functionality.

```rust
// src/symbols.rs
#[test]
fn test_dist_extra_bits() {
    // Test all valid values
    for dist in 1..=32768 {
        let c_result = unsafe { zopfli::ffi::symbols::get_dist_extra_bits(dist) };
        let rust_result = zopfli::symbols::get_dist_extra_bits(dist);
        
        assert_eq!(c_result, rust_result, 
                  "Mismatch for dist {}: C returned {}, Rust returned {}", 
                  dist, c_result, rust_result);
    }
}
```

### 8. Create Fuzz Target

Create a fuzzing target which compares your function against the C implementation.

```rust
// fuzz/fuzz_targets/fuzz_dist_extra_bits.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|dist: i32| {
    // Handle edge cases that cause panics
    if dist == i32::MIN {
        return; // abs() would overflow
    }
    
    let dist = dist.abs();
    
    // Compare C and Rust implementations
    let c_result = unsafe { zopfli::ffi::symbols::get_dist_extra_bits(dist) };
    let rust_result = zopfli::symbols::get_dist_extra_bits(dist);
    
    assert_eq!(c_result, rust_result);
});
```

Update `fuzz/Cargo.toml` to include your fuzz target.
```toml
[[bin]]
name = "fuzz_dist_extra_bits"
path = "fuzz_targets/fuzz_dist_extra_bits.rs"
test = false
doc = false
```

### 9. Test Thoroughly

1. Run unit tests:
   ```bash
   cargo test -- --nocapture
   ```

2. Run fuzzer:
   ```bash
   cd fuzz && cargo +nightly fuzz run fuzz_dist_extra_bits -- -max_total_time=10
   ```

3. If fuzzer finds issues, fix them (e.g., the `i32::MIN` case)

## Common Pitfalls and Solutions

### 1. Static Inline Functions
**Problem**: Can't link directly to static inline C functions  
**Solution**: Create wrapper functions in a `.c` file

### 2. Integer Overflow
**Problem**: Rust panics on overflow in debug mode  
**Solution**: Handle edge cases explicitly (e.g., `i32::MIN` for `abs()`)

### 3. Build Complexity
**Problem**: Trying to build the entire C library  
**Solution**: Only compile what you need (individual `.c` files or just wrappers)

### 4. Type Confusion
**Problem**: C uses platform-specific types  
**Solution**: Use `std::os::raw::*` types for FFI, convert at boundaries

### 5. Missing Dependencies
**Problem**: Function depends on other unported functions  
**Solution**: Either port dependencies first or temporarily use C versions via FFI

## Minimal Cargo.toml

Keep dependencies minimal:
```toml
[package]
name = "zopfli"
version = "0.1.0"
edition = "2021"

[features]
default = ["c-fallback"]
c-fallback = ["cc"]

[dependencies]
# No dependencies needed for basic functions

[build-dependencies]
cc = { version = "1.0", optional = true }

[dev-dependencies]
# Only what you need for testing
```

## When to Expand

Only add infrastructure when you actually need it:
- Add `types.rs` when porting structs
- Add `options.rs` when porting functions that use `ZopfliOptions`
- Add compression infrastructure only when porting compression functions

## Verification Checklist

- [ ] C and Rust produce identical results for all valid inputs
- [ ] Fuzz testing passes without panics or mismatches
- [ ] Edge cases are handled (overflow, underflow, boundary values)
- [ ] Code follows Rust idioms (no unnecessary `unsafe`, proper error handling)
- [ ] Tests are comprehensive (exhaustive where feasible, boundaries always)
- [ ] Documentation explains any non-obvious mappings from C

## Example: Complete Minimal Port

For `ZopfliGetDistExtraBits`, the complete implementation consists of:
1. `src/lib.rs` - 3 lines (module declarations)
2. `src/ffi.rs` - ~15 lines (wrapper declaration and helper)
3. `src/bridge.rs` - ~10 lines (routing function)
4. `src/symbols.rs` - ~10 lines (actual implementation)
5. `src/symbols_wrapper.c` - ~5 lines (C wrapper)
6. `build.rs` - ~15 lines (compile wrapper)
8. `fuzz/fuzz_targets/fuzz_dist_extra_bits.rs` - ~20 lines

Total: ~130 lines of code to safely port and verify one function.

Remember: Start small, test thoroughly, expand only as needed.

### Fixing bugs

When you encounter a bug at any step in this process, you must first write a new
document outlining your theory for the codebase and the bug. Write the document
into `doc/port/bugs/<timestamp_bug_name>.md`. Your document should include a paste of
the program output and expected output, followed by a description of your
understanding of how the codebase _should_ have worked and what you think went
wrong.