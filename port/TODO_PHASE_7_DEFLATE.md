# TODO: Phase 7 - DEFLATE Encoding Implementation

## Overview

Phase 6 (Block Splitting) has been successfully implemented, but it currently has a critical dependency on the DEFLATE encoding module that hasn't been ported yet. This document outlines what needs to be implemented in Phase 7 to complete the block splitting functionality and enable full Rust implementation.

## Current Limitation

The block splitting implementation in `blocksplitter.rs` has this limitation:

```rust
fn estimate_cost(lz77: &ZopfliLZ77Store, lstart: usize, lend: usize) -> f64 {
    #[cfg(not(feature = "c-fallback"))]
    {
        todo!("Deflate module not yet ported - calculate_block_size_auto_type needs implementation")
    }
}
```

This means:
- ✅ `block_split_simple()` works (no DEFLATE dependency)
- ❌ `block_split_lz77()` fails with `todo!()` in pure Rust mode
- ❌ `block_split()` fails because it calls `block_split_lz77()`
- ⚠️ C fallback mode works but defeats the purpose of the Rust port

## Required DEFLATE Functions

To fix this, Phase 7 must implement these functions from `deflate.h`:

### 1. Core Block Size Calculation
```c
double ZopfliCalculateBlockSizeAutoType(
    const ZopfliLZ77Store* lz77, 
    size_t lstart, 
    size_t lend
);
```

This is the **critical function** needed for `estimate_cost()`. It:
- Tests all three DEFLATE block types (stored, fixed Huffman, dynamic Huffman)
- Returns the size in bits of the smallest encoding
- Used by block splitting to find optimal split points

### 2. Individual Block Type Calculations
```c
double ZopfliCalculateBlockSize(
    const ZopfliLZ77Store* lz77,
    size_t lstart,
    size_t lend,
    int btype
);
```

Where `btype` is:
- 0: Stored (uncompressed)
- 1: Fixed Huffman codes
- 2: Dynamic Huffman codes

### 3. Supporting Infrastructure

From the CODEBASE_ANALYSIS.md, these functions are also needed:

#### Bit Writing Functions
- `AddBit`, `AddBits`, `AddHuffmanBits` - Low-level bit stream writing
- A Rust bitstream writer struct would be more idiomatic

#### Huffman Tree Functions  
- `GetFixedTree` - DEFLATE fixed Huffman tree bit lengths
- `GetDynamicLengths` - Calculate dynamic Huffman tree lengths
- `OptimizeHuffmanForRle` - Optimize lengths for tree encoding

#### Block Encoding Functions
- `AddNonCompressedBlock` - DEFLATE stored block (BTYPE 00)
- `AddLZ77Block` - DEFLATE compressed block (BTYPE 01 or 10)
- `AddLZ77BlockAutoType` - Choose best block type automatically

## Implementation Strategy

### Phase 7A: Minimum Viable Implementation
Focus only on what's needed for block splitting:

1. **Implement `ZopfliCalculateBlockSizeAutoType`**
   - Start with a simplified version that calculates rough estimates
   - Can improve accuracy later

2. **Implement basic block size calculation for each type**
   - Stored: `8 * (lend - lstart)` bits (uncompressed size)
   - Fixed: Use static bit costs for symbols
   - Dynamic: Use entropy-based estimates initially

3. **Update `blocksplitter.rs`**
   ```rust
   fn estimate_cost(lz77: &ZopfliLZ77Store, lstart: usize, lend: usize) -> f64 {
       crate::deflate::calculate_block_size_auto_type(lz77, lstart, lend)
   }
   ```

### Phase 7B: Complete Implementation
Implement full DEFLATE encoding for compression pipeline:

1. **Bit stream writer**
2. **Complete Huffman tree handling**  
3. **Full block encoding with exact bit costs**
4. **Integration with container formats (gzip/zlib)**

## Testing Requirements

Once Phase 7A is complete:

1. **Update block splitting fuzzer**
   - Remove the `#[cfg(not(feature = "c-fallback"))]` guards
   - Enable full testing of `block_split_lz77()` and `block_split()`

2. **Add DEFLATE-specific tests**
   - Verify bit costs match C implementation
   - Test all three block types
   - Ensure block splitting finds optimal points

3. **Integration testing**
   - Test end-to-end compression with block splitting
   - Verify compression ratios match C implementation

## Files to Modify

### New Files
- `zopfli-rs/src/deflate.rs` - Main DEFLATE module
- `zopfli-rs/src/bitwriter.rs` - Bit stream writing utilities (optional)

### Existing Files to Update
- `zopfli-rs/src/blocksplitter.rs` - Remove `todo!()` from `estimate_cost()`
- `zopfli-rs/src/lib.rs` - Add `pub mod deflate;`
- `zopfli-rs/fuzz/fuzz_targets/fuzz_blocksplitter.rs` - Enable full testing
- `zopfli-rs/src/bridge.rs` - Add deflate bridge functions

## Success Criteria

Phase 7A is complete when:
- ✅ `block_split_lz77()` works in pure Rust mode (no `todo!()`)
- ✅ `block_split()` works in pure Rust mode  
- ✅ Block splitting fuzzer passes with C/Rust comparison
- ✅ Block splitting produces reasonable split points (not necessarily optimal)

Phase 7B is complete when:
- ✅ Full DEFLATE encoding matches C implementation exactly
- ✅ Compression ratios match C implementation
- ✅ All DEFLATE block types implemented correctly

## Priority

**HIGH** - Phase 6 is incomplete without this. Block splitting is a core Zopfli optimization that significantly improves compression ratios. The current implementation works only in C fallback mode, defeating the purpose of the Rust port.

## Estimated Effort

- **Phase 7A (minimum viable)**: 1-2 days
- **Phase 7B (complete)**: 1-2 weeks

The minimum viable implementation should be prioritized to unblock Phase 6 testing and validation.