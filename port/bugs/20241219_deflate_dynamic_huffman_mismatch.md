# Bug Report: DEFLATE Dynamic Huffman Block Size Calculation Mismatch

**Date:** 2024-12-19  
**Component:** DEFLATE dynamic Huffman tree encoding  
**Severity:** High  
**Status:** DISCOVERED  

## Problem Description

The Rust implementation of `calculate_block_size` for dynamic Huffman blocks (btype=2) produces different results compared to the C implementation. Specifically:

- **Expected (C)**: 308.00 bits
- **Actual (Rust)**: 309.00 bits
- **Input data**: `[65, 68, 223, 205, 202, 120, 32, 122, 219, 64, 135, 0, 8, 255, 0, 0, 0, 0, 0, 164, 35]`

## Reproduction

The discrepancy was discovered through fuzzing:

```bash
cd /Users/power/code/zopfli/zopfli-rs/fuzz
cargo +nightly fuzz run fuzz_deflate -- -max_total_time=30
```

The failing input can be reproduced with:
```bash
cargo fuzz run fuzz_deflate artifacts/fuzz_deflate/crash-77baefebe82adebc6a7e0c3cd028a63d08cc3cdd
```

## Root Cause Analysis

The discrepancy occurs in the dynamic Huffman tree size calculation. This could be due to:

1. **Tree encoding differences**: The `encode_tree` function may be calculating tree sizes differently than the C version
2. **RLE optimization**: Differences in `optimize_huffman_for_rle` implementation
3. **Huffman bit length calculation**: Subtle differences in `calculate_bit_lengths` for edge cases
4. **Floating point precision**: Minor differences in tree size calculation

## Investigation Plan

1. **Test each component separately**:
   - Compare `get_dynamic_lengths` outputs between C and Rust
   - Compare `calculate_tree_size` outputs 
   - Compare `calculate_block_symbol_size` outputs

2. **Check intermediate values**:
   - Compare histogram counts (`ll_counts`, `d_counts`)
   - Compare bit lengths arrays (`ll_lengths`, `d_lengths`)
   - Compare tree encoding parameters (use_16, use_17, use_18)

3. **Verify RLE optimization**:
   - Test `optimize_huffman_for_rle` with the specific input
   - Compare `try_optimize_huffman_for_rle` results

## Temporary Workaround

None - this must be fixed for exact C compatibility.

## Impact

- Breaks exact compatibility with C implementation
- May indicate deeper algorithmic differences
- Could affect compression ratios in edge cases
- Prevents successful fuzzing completion

## Next Steps

1. Add debugging output to identify where the divergence occurs
2. Implement step-by-step comparison between C and Rust intermediate values
3. Fix the root cause to achieve exact compatibility
4. Re-run fuzzing to ensure no other discrepancies exist