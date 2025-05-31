/*!
Utilities for creating and using Huffman trees.

This module implements the Katajainen algorithm for length-limited Huffman coding
and other tree-related utilities.
*/


/// Node structure for the Katajainen algorithm
#[derive(Debug, Clone)]
struct Node {
    /// Total weight (symbol count) of this chain
    weight: usize,
    /// Previous node(s) of this chain, or None if none
    tail: Option<Box<Node>>,
    /// Leaf symbol index, or number of leaves before this chain
    count: i32,
}

impl Node {
    fn new(weight: usize, count: i32, tail: Option<Box<Node>>) -> Self {
        Node { weight, count, tail }
    }
}

/// Memory pool for nodes (simplified version using Vec)
struct NodePool {
    nodes: Vec<Node>,
}

impl NodePool {
    fn new(capacity: usize) -> Self {
        NodePool {
            nodes: Vec::with_capacity(capacity),
        }
    }
    
    fn allocate(&mut self, weight: usize, count: i32, tail: Option<Box<Node>>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node::new(weight, count, tail));
        index
    }
    
    fn get(&self, index: usize) -> &Node {
        &self.nodes[index]
    }
    
    fn get_mut(&mut self, index: usize) -> &mut Node {
        &mut self.nodes[index]
    }
}

/// Outputs minimum-redundancy length-limited code bitlengths for symbols with the
/// given counts. The bitlengths are limited by maxbits.
///
/// The output is tailored for DEFLATE: symbols that never occur, get a bit length
/// of 0, and if only a single symbol occurs at least once, its bitlength will be 1,
/// and not 0 as would theoretically be needed for a single symbol.
///
/// Returns Ok(()) on success, Err(()) on error.
pub fn length_limited_code_lengths(
    frequencies: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    let _n = frequencies.len();
    
    // Initialize all bitlengths at 0
    for bitlength in bitlengths.iter_mut() {
        *bitlength = 0;
    }
    
    // Count used symbols and create leaves
    let mut leaves = Vec::new();
    for (i, &freq) in frequencies.iter().enumerate() {
        if freq > 0 {
            leaves.push(Node::new(freq, i as i32, None));
        }
    }
    
    let numsymbols = leaves.len();
    
    // Check special cases and error conditions
    if (1 << maxbits) < numsymbols {
        return Err(()); // Too few maxbits to represent symbols
    }
    if numsymbols == 0 {
        return Ok(()); // No symbols at all
    }
    if numsymbols == 1 {
        bitlengths[leaves[0].count as usize] = 1;
        return Ok(()); // Only one symbol, give it bitlength 1
    }
    if numsymbols == 2 {
        bitlengths[leaves[0].count as usize] = 1;
        bitlengths[leaves[1].count as usize] = 1;
        return Ok(());
    }
    
    // Sort the leaves from lightest to heaviest with stable sorting
    for leaf in &mut leaves {
        if leaf.weight >= (1_usize << (std::mem::size_of::<usize>() * 8 - 9)) {
            return Err(()); // Need 9 bits for the count
        }
        leaf.weight = (leaf.weight << 9) | (leaf.count as usize);
    }
    
    leaves.sort_by(|a, b| a.weight.cmp(&b.weight));
    
    for leaf in &mut leaves {
        leaf.weight >>= 9;
    }
    
    let maxbits = if (numsymbols - 1) < maxbits as usize {
        numsymbols - 1
    } else {
        maxbits as usize
    };
    
    // Initialize node memory pool
    let mut pool = NodePool::new(maxbits * 2 * numsymbols);
    
    // Array of lists of chains. Each list requires only two lookahead chains
    let mut lists = vec![(0usize, 0usize); maxbits];
    
    // Initialize lists with two leaves with lowest weights
    let node0_idx = pool.allocate(leaves[0].weight, 1, None);
    let node1_idx = pool.allocate(leaves[1].weight, 2, None);
    
    for i in 0..maxbits {
        lists[i] = (node0_idx, node1_idx);
    }
    
    // Boundary package-merge runs
    let num_boundary_pm_runs = 2 * numsymbols - 4;
    for _ in 0..num_boundary_pm_runs - 1 {
        boundary_pm(&mut lists, &leaves, numsymbols, &mut pool, maxbits - 1);
    }
    boundary_pm_final(&mut lists, &leaves, numsymbols, &mut pool, maxbits - 1);
    
    // Extract bit lengths
    extract_bit_lengths(&pool, lists[maxbits - 1].1, &leaves, bitlengths);
    
    Ok(())
}

/// Performs a Boundary Package-Merge step
fn boundary_pm(
    lists: &mut [(usize, usize)],
    leaves: &[Node],
    numsymbols: usize,
    pool: &mut NodePool,
    index: usize,
) {
    let lastcount = pool.get(lists[index].1).count;
    
    if index == 0 && lastcount >= numsymbols as i32 {
        return;
    }
    
    let oldchain_idx = lists[index].1;
    
    if index == 0 {
        // New leaf node in list 0
        let newchain_idx = pool.allocate(
            leaves[lastcount as usize].weight,
            lastcount + 1,
            None,
        );
        lists[index] = (oldchain_idx, newchain_idx);
    } else {
        let sum = pool.get(lists[index - 1].0).weight + pool.get(lists[index - 1].1).weight;
        
        if lastcount < numsymbols as i32 && sum > leaves[lastcount as usize].weight {
            // New leaf inserted in list
            let oldchain_tail = if let Some(ref tail) = pool.get(oldchain_idx).tail {
                Some(tail.clone())
            } else {
                None
            };
            
            let newchain_idx = pool.allocate(
                leaves[lastcount as usize].weight,
                lastcount + 1,
                oldchain_tail,
            );
            lists[index] = (oldchain_idx, newchain_idx);
        } else {
            let tail_node = Box::new(pool.get(lists[index - 1].1).clone());
            let newchain_idx = pool.allocate(sum, lastcount, Some(tail_node));
            lists[index] = (oldchain_idx, newchain_idx);
            
            // Two lookahead chains of previous list used up
            boundary_pm(lists, leaves, numsymbols, pool, index - 1);
            boundary_pm(lists, leaves, numsymbols, pool, index - 1);
        }
    }
}

/// Final boundary package-merge step
fn boundary_pm_final(
    lists: &mut [(usize, usize)],
    leaves: &[Node],
    numsymbols: usize,
    pool: &mut NodePool,
    index: usize,
) {
    let lastcount = pool.get(lists[index].1).count;
    let sum = pool.get(lists[index - 1].0).weight + pool.get(lists[index - 1].1).weight;
    
    if lastcount < numsymbols as i32 && sum > leaves[lastcount as usize].weight {
        let oldchain_tail = if let Some(ref tail) = pool.get(lists[index].1).tail {
            Some(tail.clone())
        } else {
            None
        };
        
        let newchain_idx = pool.allocate(0, lastcount + 1, oldchain_tail); // weight doesn't matter here
        lists[index].1 = newchain_idx;
    } else {
        let tail_node = Box::new(pool.get(lists[index - 1].1).clone());
        pool.get_mut(lists[index].1).tail = Some(tail_node);
    }
}

/// Extracts bit lengths from the final chain
fn extract_bit_lengths(
    pool: &NodePool,
    chain_idx: usize,
    leaves: &[Node],
    bitlengths: &mut [u32],
) {
    let mut counts = [0i32; 16];
    let mut end = 16;
    let mut ptr = 15;
    let mut value = 1u32;
    
    // Walk the chain and collect counts
    let mut current_node = Some(pool.get(chain_idx));
    while let Some(node) = current_node {
        end -= 1;
        counts[end] = node.count;
        current_node = node.tail.as_ref().map(|boxed| boxed.as_ref());
    }
    
    let mut val = counts[15];
    while ptr >= end {
        while val > counts[ptr - 1] {
            val -= 1;
            bitlengths[leaves[val as usize].count as usize] = value;
        }
        ptr -= 1;
        value += 1;
    }
}

/// Calculates the bitlengths for the Huffman tree, based on the counts of each symbol.
pub fn calculate_bit_lengths(
    count: &[usize],
    maxbits: i32,
    bitlengths: &mut [u32],
) -> Result<(), ()> {
    length_limited_code_lengths(count, maxbits, bitlengths)
}

/// Converts a series of Huffman tree bitlengths, to the bit values of the symbols.
pub fn lengths_to_symbols(
    lengths: &[u32],
    maxbits: u32,
    symbols: &mut [u32],
) {
    let _n = lengths.len();
    
    // Initialize symbols to 0
    for symbol in symbols.iter_mut() {
        *symbol = 0;
    }
    
    let mut bl_count = vec![0usize; (maxbits + 1) as usize];
    let mut next_code = vec![0usize; (maxbits + 1) as usize];
    
    // Count the number of codes for each code length
    for &length in lengths {
        assert!(length <= maxbits);
        bl_count[length as usize] += 1;
    }
    
    // Find the numerical value of the smallest code for each code length
    let mut code = 0usize;
    bl_count[0] = 0;
    for bits in 1..=(maxbits as usize) {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }
    
    // Assign numerical values to all codes
    for (i, &len) in lengths.iter().enumerate() {
        if len != 0 {
            symbols[i] = next_code[len as usize] as u32;
            next_code[len as usize] += 1;
        }
    }
}

/// Calculates the entropy of each symbol, based on the counts of each symbol.
/// The result is similar to the result of calculate_bit_lengths, but with the
/// actual theoretical bit lengths according to the entropy.
pub fn calculate_entropy(count: &[usize], bitlengths: &mut [f64]) {
    const INV_LOG2: f64 = 1.4426950408889; // 1.0 / log(2.0)
    
    let sum: usize = count.iter().sum();
    let log2sum = if sum == 0 {
        (count.len() as f64).ln() * INV_LOG2
    } else {
        (sum as f64).ln() * INV_LOG2
    };
    
    for (i, &cnt) in count.iter().enumerate() {
        if cnt == 0 {
            bitlengths[i] = log2sum;
        } else {
            bitlengths[i] = log2sum - (cnt as f64).ln() * INV_LOG2;
        }
        
        // Clamp negative values very close to zero
        if bitlengths[i] < 0.0 && bitlengths[i] > -1e-5 {
            bitlengths[i] = 0.0;
        }
        
        assert!(bitlengths[i] >= 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_length_limited_code_lengths_basic() {
        let frequencies = [5, 9, 12, 13, 16, 45];
        let mut bitlengths = [0u32; 6];
        
        let result = length_limited_code_lengths(&frequencies, 15, &mut bitlengths);
        assert!(result.is_ok());
        
        // Verify all symbols got some bit length
        for (i, &freq) in frequencies.iter().enumerate() {
            if freq > 0 {
                assert!(bitlengths[i] > 0, "Symbol {} with frequency {} got bit length 0", i, freq);
            }
        }
    }
    
    #[test]
    fn test_lengths_to_symbols() {
        let lengths = [3, 3, 3, 3, 3, 2, 4, 4];
        let mut symbols = [0u32; 8];
        
        lengths_to_symbols(&lengths, 4, &mut symbols);
        
        // Verify canonical Huffman codes
        assert_eq!(symbols[5], 0); // length 2, first code
        assert_eq!(symbols[0], 2); // length 3, codes 2-7
        assert_eq!(symbols[6], 14); // length 4, codes 14-15
    }
    
    #[test]
    fn test_calculate_entropy() {
        let count = [5, 9, 12, 13, 16, 45];
        let mut bitlengths = [0.0; 6];
        
        calculate_entropy(&count, &mut bitlengths);
        
        // Entropy should be non-negative and reasonable
        for &bitlength in &bitlengths {
            assert!(bitlength >= 0.0);
            assert!(bitlength < 10.0); // Sanity check
        }
        
        // Higher frequency symbols should have lower entropy
        assert!(bitlengths[5] < bitlengths[0]); // 45 < 5
    }
}