# Bug Report: LZ77 Implementation Hash Initialization Issue

**Date:** 2024-12-19
**Component:** LZ77 longest match finding
**Severity:** Critical
**Status:** RESOLVED

## Problem Description

The Rust implementation of `find_longest_match` was appearing to return incorrect results compared to the C implementation. Specifically:

- Expected: Should find matches between repeated patterns in data
- Actual: Always returned `length=1, distance=0` (no matches found)

## Root Cause

The issue was not with the Rust implementation itself, but with how the hash table was being initialized in tests and fuzzing. The hash table must be initialized to match the state it would be in during normal sequential processing up to the position being tested.

### The Problem

When testing `find_longest_match` at position 3 with data "abcabcabc":
- **Incorrect**: Initializing hash for all positions 0-8, then testing position 3
- **Correct**: Initializing hash only for positions 0-3, then testing position 3

The C wrapper had the same issue, which caused the assertion `pp == hpos` to fail.

## Resolution

1. **Fixed the fuzzer** to properly initialize hash state for each test position
2. **Fixed the test** to initialize hash only up to the position being tested
3. **Removed unnecessary C wrapper** and used direct FFI bindings
4. **Added mutable cache update methods** to properly update the cache

## Verification

After fixing the hash initialization:
- Test case "abcabcabc" at position 3 correctly finds length=6, distance=3
- Fuzzer runs successfully without C/Rust mismatches
- All LZ77 functions pass comprehensive fuzzing

## Code Changes

### Fixed Test
```rust
// Initialize hash with the data up to position 3
// This simulates the state the hash would be in during normal processing
hash.reset(ZOPFLI_WINDOW_SIZE);
hash.warmup(data, 0, 4); // warmup to position 3 + 1
for i in 0..=3 {
    hash.update(data, i, data.len());
}
```

### Fixed Cache API
```rust
pub fn update_length_dist(&mut self, pos: usize, length: u16, dist: u16) {
    if pos < self.length.len() {
        self.length[pos] = length;
        self.dist[pos] = dist;
    }
}
```

## Lessons Learned

When porting compression algorithms that maintain state during processing, it's crucial to ensure that test cases properly simulate the incremental state building that occurs during normal operation. The hash table in Zopfli is built incrementally as data is processed, and testing a specific position requires recreating the exact state the hash would be in at that point.