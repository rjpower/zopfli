use crate::deflate::*;
use crate::lz77::*;
use crate::options::ZopfliOptions;
use crate::hash::ZopfliHash;
use crate::util::*;
use crate::symbols::*;

/// Debug function to reproduce the exact bug found by fuzzing
pub fn debug_dynamic_huffman_bug() {
    // Failing input from fuzzer
    let data = [63, 34, 34, 34, 223, 221, 255, 255, 255, 255, 255, 254, 0, 0, 0, 34, 34, 34, 34, 34, 34, 34, 34, 0, 0, 0, 0, 0, 37];
    
    // Create LZ77 representation
    let mut rust_store = ZopfliLZ77Store::new(&data);
    let options = ZopfliOptions::default();
    let mut s = ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
    let mut hash = ZopfliHash::new(crate::util::ZOPFLI_WINDOW_SIZE);
    
    // Create LZ77 representation
    lz77_greedy(&mut s, &data, 0, data.len(), &mut rust_store, &mut hash);
    
    if rust_store.size() == 0 {
        println!("Empty store - skipping");
        return;
    }

    let lstart = 0;
    let lend = rust_store.size();
    
    println!("Testing input: {:?}", data);
    println!("LZ77 store size: {}", rust_store.size());
    
    // Calculate Rust dynamic Huffman block size
    let rust_size = calculate_block_size(&rust_store, lstart, lend, 2);
    println!("Rust dynamic block size: {:.2}", rust_size);
    
    // Now let's debug the intermediate steps
    debug_dynamic_lengths(&rust_store, lstart, lend);
}

/// Debug the get_dynamic_lengths function step by step
fn debug_dynamic_lengths(lz77: &ZopfliLZ77Store, lstart: usize, lend: usize) {
    use crate::tree::calculate_bit_lengths;
    
    println!("\n=== Debugging get_dynamic_lengths ===");
    
    // Step 1: Get histogram
    let mut ll_counts = [0usize; ZOPFLI_NUM_LL];
    let mut d_counts = [0usize; ZOPFLI_NUM_D];
    
    lz77.get_histogram(lstart, lend, &mut ll_counts, &mut d_counts);
    ll_counts[256] = 1; // End symbol
    
    println!("LL counts (non-zero):");
    for i in 0..ZOPFLI_NUM_LL {
        if ll_counts[i] > 0 {
            println!("  [{}] = {}", i, ll_counts[i]);
        }
    }
    
    println!("D counts (non-zero):");
    for i in 0..ZOPFLI_NUM_D {
        if d_counts[i] > 0 {
            println!("  [{}] = {}", i, d_counts[i]);
        }
    }
    
    // Step 2: Calculate bit lengths
    let mut ll_lengths = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths = [0u32; ZOPFLI_NUM_D];
    
    let _ = calculate_bit_lengths(&ll_counts, 15, &mut ll_lengths);
    let _ = calculate_bit_lengths(&d_counts, 15, &mut d_lengths);
    
    println!("\nLL lengths (non-zero):");
    for i in 0..ZOPFLI_NUM_LL {
        if ll_lengths[i] > 0 {
            println!("  [{}] = {}", i, ll_lengths[i]);
        }
    }
    
    println!("D lengths (non-zero):");
    for i in 0..ZOPFLI_NUM_D {
        if d_lengths[i] > 0 {
            println!("  [{}] = {}", i, d_lengths[i]);
        }
    }
    
    // Step 3: Patch distance codes
    patch_distance_codes_for_buggy_decoders(&mut d_lengths);
    
    println!("\nD lengths after patching (non-zero):");
    for i in 0..ZOPFLI_NUM_D {
        if d_lengths[i] > 0 {
            println!("  [{}] = {}", i, d_lengths[i]);
        }
    }
    
    // Step 4: Calculate costs
    let tree_size = calculate_tree_size(&ll_lengths, &d_lengths);
    let data_size = calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, &ll_lengths, &d_lengths, lz77, lstart, lend);
    
    println!("\nTree size: {:.2}", tree_size);
    println!("Data size: {}", data_size);
    println!("Total (before RLE): {:.2}", tree_size + data_size as f64);
    
    // Step 5: Try RLE optimization
    let mut ll_counts2 = ll_counts.clone();
    let mut d_counts2 = d_counts.clone();
    optimize_huffman_for_rle(ZOPFLI_NUM_LL, &mut ll_counts2);
    optimize_huffman_for_rle(ZOPFLI_NUM_D, &mut d_counts2);
    
    println!("\nAfter RLE optimization:");
    println!("LL counts changed: {}", ll_counts != ll_counts2);
    println!("D counts changed: {}", d_counts != d_counts2);
    
    let mut ll_lengths2 = [0u32; ZOPFLI_NUM_LL];
    let mut d_lengths2 = [0u32; ZOPFLI_NUM_D];
    let _ = calculate_bit_lengths(&ll_counts2, 15, &mut ll_lengths2);
    let _ = calculate_bit_lengths(&d_counts2, 15, &mut d_lengths2);
    patch_distance_codes_for_buggy_decoders(&mut d_lengths2);
    
    let tree_size2 = calculate_tree_size(&ll_lengths2, &d_lengths2);
    let data_size2 = calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, &ll_lengths2, &d_lengths2, lz77, lstart, lend);
    
    println!("Tree size (RLE): {:.2}", tree_size2);
    println!("Data size (RLE): {}", data_size2);
    println!("Total (RLE): {:.2}", tree_size2 + data_size2 as f64);
    
    // Final choice
    let final_cost = if tree_size2 + (data_size2 as f64) < tree_size + (data_size as f64) {
        println!("\nChose RLE optimization");
        tree_size2 + data_size2 as f64
    } else {
        println!("\nChose original (no RLE)");
        tree_size + data_size as f64
    };
    
    println!("Final cost: {:.2}", final_cost);
    println!("With 3-bit header: {:.2}", 3.0 + final_cost);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_huffman_bug() {
        debug_dynamic_huffman_bug();
    }
}