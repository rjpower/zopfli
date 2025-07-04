# Single Function/Struct Porting Guide for Zopfli

This guide provides practical instructions for porting a single function or
struct from the Zopfli C codebase to Rust.

You will be given one or more tasks from @port/TASKS.md.


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
```

## Key Principles

1. **Minimize scope**: Only implement what's needed for the specific function/struct being ported
2. **Test exhaustively**: Use both comprehensive unit tests and fuzzing
3. **Handle C quirks**: inline functions need wrappers, watch for integer overflow edge cases
4. **Keep it simple**: Don't build infrastructure you don't need yet
5. Don't include COPYRIGHT headers!

## Step-by-Step Porting Process

### 1. Analyze the C Function/Struct

First, understand what you're porting:
- Check if it's a static inline function (requires wrapper)
- Identify dependencies (other functions/structs it uses)
- Note any macros or constants used
- Check for platform-specific code or undefined behavior
- **Look for conditional compilation** (`#ifdef` blocks) and check @port/CODEBASE_ANALYSIS.md for which flags are active
- **Identify complex algorithms** that may have edge cases (anything with loops, recursion, or assertions)

Example findings for `ZopfliGetDistExtraBits`:
- Static inline function in `symbols.h`
- Uses `__builtin_clz` (maps to `leading_zeros()` in Rust)
- Takes `int`, returns `int`
- No dependencies on other functions

Example findings for `ZopfliHash`:
- Regular exported functions, no wrappers needed
- Multiple conditional compilation blocks (`ZOPFLI_HASH_SAME`, `ZOPFLI_HASH_SAME_HASH`)
- Complex state with multiple arrays and rolling hash algorithm
- Requires careful bounds checking and window size management

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

Update `build.rs` as needed:
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

Declare the C functions or structures you need for your task in `src/ffi.rs`.

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

Create a focused implementation, with a goal of being as close to the C
implementation as possible.  Follow the guidelines from @port/RUST_PORTING.md
for how to write good Rust code.

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

Create an inline test suite for your function, testing basic functionality.

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

Create a fuzzing target. This is critical, as it compares your function against
the reference C implementation.

```rust
// fuzz/fuzz_targets/fuzz_symbols.rs
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
name = "fuzz_symbols"
path = "fuzz_targets/fuzz_symbols.rs"
test = false
doc = false
```

### 9. Test Thoroughly

1. Run unit tests:
   ```bash
   cargo test -- --nocapture
   ```

2. **Run fuzzer EARLY and for complex functions**: 
   ```bash
   cd fuzz && cargo +nightly fuzz run fuzz_symbols -- -max_total_time=30
   ```
   
3. For complex structs/algorithms, run fuzzer for longer (e.g., 60+ seconds)

4. If fuzzer finds issues:
   - **Document the edge case** in comments
   - **Handle gracefully** rather than asserting  
   - **Test the fix** with specific unit tests

## Common Pitfalls and Solutions

### 1. Static Inline Functions
**Problem**: Can't link directly to static inline C functions  
**Solution**: Create wrapper functions in a `.c` file

### 2. Conditional Compilation Complexity
**Problem**: C code has many `#ifdef` blocks that are hard to understand  
**Solution**: Check @port/CODEBASE_ANALYSIS.md which documents which flags are assumed active

### 3. Edge Cases in Assertions
**Problem**: C code has assertions that may fail on edge cases (like distance-0 in caches)  
**Solution**: Run fuzzing early to discover edge cases, then handle them gracefully

### 4. Complex Struct Memory Layout  
**Problem**: C structs with conditional fields or complex allocation patterns  
**Solution**: Always include ALL fields in Rust structs, even if conditionally compiled in C

### 5. Debugging Assertion Failures
**Problem**: Hard to understand why assertions fail in complex algorithms  
**Solution**: 
- Use `debug_assert!` instead of `assert!` for performance-critical code
- Add detailed error messages with context
- Consider making assertions conditional when edge cases are acceptable

## When to Expand

Only add infrastructure when you actually need it:
- Add `types.rs` when porting structs
- Add `options.rs` when porting functions that use `ZopfliOptions`
- Add compression infrastructure only when porting compression functions

## Complex Struct Porting Strategy

For structs with many fields and complex behavior (like `ZopfliHash`, `ZopfliLongestMatchCache`):

### 1. Start with Complete Struct Definition
```rust
// Include ALL fields, even those conditionally compiled in C
#[derive(Debug)]  // Always add Debug for easier debugging
pub struct ZopfliHash {
    // All main fields
    head: Vec<i32>,
    prev: Vec<u16>,
    // All conditional compilation fields  
    head2: Vec<i32>,  // ZOPFLI_HASH_SAME_HASH
    same: Vec<u16>,   // ZOPFLI_HASH_SAME
}
```

### 2. Implement Core Methods First
- `new()` - Constructor with proper initialization
- `Drop` - Automatic cleanup (usually just `Vec` drop)
- Basic operations without complex logic

### 3. Add Complex Methods Incrementally  
- Implement one method at a time
- **Write unit tests for each method**
- **Run fuzzer after each complex method**

### 4. FFI Integration Last
- Add `#[repr(C)]` struct definitions
- Create bridge structs that handle both C and Rust
- Test FFI integration separately

### 5. Handle Edge Cases Discovered by Fuzzing
- **Document edge cases** in code comments
- Make assertions conditional when appropriate:
```rust
// Handle edge case where all distances are 0
if self.max_cached_sublen(pos, length) > 0 {
    debug_assert_eq!(bestlength, self.max_cached_sublen(pos, length));
}
```


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

### Fixing bugs

When you encounter a bug at any step in this process, you must first write a new
document outlining your theory for the codebase and the bug. Write the document
into `doc/port/bugs/<timestamp_bug_name>.md`. Your document should include a paste of
the program output and expected output, followed by a description of your
understanding of how the codebase _should_ have worked and what you think went
wrong.

### 10. Update TASKS.md and commit

After you have completed the porting, update the TASKS.md file to mark the task as completed.
Then commit your changes with an appropriate commit message.