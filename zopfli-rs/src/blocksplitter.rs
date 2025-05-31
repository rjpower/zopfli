use crate::util::ZOPFLI_LARGE_FLOAT;
use crate::options::ZopfliOptions;
use crate::lz77::ZopfliLZ77Store;
use crate::hash::ZopfliHash;
use crate::lz77::ZopfliBlockState;

/// Estimate cost of a block using block size calculation.
/// Returns estimated cost of a block in bits. It includes the size to encode the
/// tree and the size to encode all literal, length and distance symbols and their
/// extra bits.
fn estimate_cost(lz77: &ZopfliLZ77Store, lstart: usize, lend: usize) -> f64 {
    // For now, we need to route this to the C implementation since we haven't
    // ported the deflate module yet. This will be replaced with the Rust implementation
    // once deflate.rs is available.
    #[cfg(feature = "c-fallback")]
    unsafe {
        // We need to create a temporary C store that owns the data
        let litlens: Vec<u16> = lz77.litlens().to_vec();
        let dists: Vec<u16> = lz77.dists().to_vec();
        let pos: Vec<usize> = lz77.pos().to_vec();
        let ll_symbol: Vec<u16> = lz77.ll_symbol().to_vec();
        let d_symbol: Vec<u16> = lz77.d_symbol().to_vec();
        let ll_counts: Vec<usize> = lz77.ll_counts().to_vec();
        let d_counts: Vec<usize> = lz77.d_counts().to_vec();
        
        let c_store = crate::ffi::ZopfliLZ77StoreC {
            litlens: litlens.as_ptr() as *mut u16,
            dists: dists.as_ptr() as *mut u16,
            size: lz77.size(),
            data: lz77.data().as_ptr(),
            pos: pos.as_ptr() as *mut usize,
            ll_symbol: ll_symbol.as_ptr() as *mut u16,
            d_symbol: d_symbol.as_ptr() as *mut u16,
            ll_counts: ll_counts.as_ptr() as *mut usize,
            d_counts: d_counts.as_ptr() as *mut usize,
        };
        
        let result = crate::ffi::deflate::calculate_block_size_auto_type(
            &c_store as *const _, 
            lstart, 
            lend
        );
        result
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        // This will be implemented when deflate.rs is ported
        todo!("Deflate module not yet ported - calculate_block_size_auto_type needs implementation")
    }
}

/// Context structure for split cost callback function
struct SplitCostContext<'a> {
    lz77: &'a ZopfliLZ77Store<'a>,
    start: usize,
    end: usize,
}

/// Gets the cost which is the sum of the cost of the left and the right section
/// of the data. This is the callback function for FindMinimum.
fn split_cost(i: usize, context: &SplitCostContext) -> f64 {
    estimate_cost(context.lz77, context.start, i) + estimate_cost(context.lz77, i, context.end)
}

/// Finds minimum of function f(i) where i is of type usize, f(i) is of type
/// f64, i is in range start-end (excluding end).
/// Returns the index of the minimum value, and sets *smallest to the minimum value.
fn find_minimum<F>(f: F, start: usize, end: usize) -> (usize, f64)
where
    F: Fn(usize) -> f64,
{
    if end - start < 1024 {
        let mut best = ZOPFLI_LARGE_FLOAT;
        let mut result = start;
        
        for i in start..end {
            let v = f(i);
            if v < best {
                best = v;
                result = i;
            }
        }
        
        (result, best)
    } else {
        // Try to find minimum faster by recursively checking multiple points.
        const NUM: usize = 9; // Good value: 9.
        
        let mut start = start;
        let mut end = end;
        let mut lastbest = ZOPFLI_LARGE_FLOAT;
        let mut pos = start;
        
        loop {
            if end - start <= NUM {
                break;
            }
            
            let mut p = [0usize; NUM];
            let mut vp = [0.0f64; NUM];
            
            for i in 0..NUM {
                p[i] = start + (i + 1) * ((end - start) / (NUM + 1));
                vp[i] = f(p[i]);
            }
            
            let mut besti = 0;
            let mut best = vp[0];
            for i in 1..NUM {
                if vp[i] < best {
                    best = vp[i];
                    besti = i;
                }
            }
            
            if best > lastbest {
                break;
            }
            
            start = if besti == 0 { start } else { p[besti - 1] };
            end = if besti == NUM - 1 { end } else { p[besti + 1] };
            
            pos = p[besti];
            lastbest = best;
        }
        
        (pos, lastbest)
    }
}

/// Adds a value to a sorted vector, maintaining sort order
fn add_sorted(value: usize, out: &mut Vec<usize>) {
    out.push(value);
    
    // Insert in sorted position
    for i in 0..(out.len() - 1) {
        if out[i] > value {
            // Shift elements right and insert at position i
            for j in (i + 1..out.len()).rev() {
                out[j] = out[j - 1];
            }
            out[i] = value;
            break;
        }
    }
}

/// Finds next block to try to split, the largest of the available ones.
/// The largest is chosen to make sure that if only a limited amount of blocks is
/// requested, their sizes are spread evenly.
/// 
/// Returns Some((lstart, lend)) if a block was found, None if no block found.
fn find_largest_splittable_block(
    lz77size: usize,
    done: &[bool],
    splitpoints: &[usize]
) -> Option<(usize, usize)> {
    let mut longest = 0;
    let mut result = None;
    
    for i in 0..=splitpoints.len() {
        let start = if i == 0 { 0 } else { splitpoints[i - 1] };
        let end = if i == splitpoints.len() { lz77size - 1 } else { splitpoints[i] };
        
        // Ensure start is within bounds of the done array
        if start < done.len() && !done[start] && end - start > longest {
            result = Some((start, end));
            longest = end - start;
        }
    }
    
    result
}

/// Does blocksplitting on LZ77 data.
/// The output splitpoints are indices in the LZ77 data.
/// maxblocks: set a limit to the amount of blocks. Set to 0 to mean no limit.
pub fn block_split_lz77(
    options: &ZopfliOptions,
    lz77: &ZopfliLZ77Store,
    maxblocks: usize
) -> Vec<usize> {
    if lz77.size() < 10 {
        return Vec::new(); // This code fails on tiny files.
    }
    
    let mut splitpoints = Vec::new();
    let mut done = vec![false; lz77.size()];
    let mut numblocks = 1;
    
    let mut lstart = 0;
    let mut lend = lz77.size();
    
    loop {
        if maxblocks > 0 && numblocks >= maxblocks {
            break;
        }
        
        let context = SplitCostContext {
            lz77,
            start: lstart,
            end: lend,
        };
        
        assert!(lstart < lend);
        let (llpos, splitcost) = find_minimum(
            |i| split_cost(i, &context),
            lstart + 1,
            lend
        );
        
        assert!(llpos > lstart);
        assert!(llpos < lend);
        
        let origcost = estimate_cost(lz77, lstart, lend);
        
        if splitcost > origcost || llpos == lstart + 1 || llpos == lend {
            done[lstart] = true;
        } else {
            add_sorted(llpos, &mut splitpoints);
            numblocks += 1;
        }
        
        if let Some((new_lstart, new_lend)) = find_largest_splittable_block(
            lz77.size(), &done, &splitpoints
        ) {
            lstart = new_lstart;
            lend = new_lend;
        } else {
            break; // No further split will probably reduce compression.
        }
        
        if lend - lstart < 10 {
            break;
        }
    }
    
    if options.verbose != 0 {
        eprintln!("block split points: {:?}", splitpoints);
    }
    
    splitpoints
}

/// Does blocksplitting on uncompressed data.
/// The output splitpoints are indices in the uncompressed bytes.
pub fn block_split(
    options: &ZopfliOptions,
    input: &[u8],
    instart: usize,
    inend: usize,
    maxblocks: usize
) -> Vec<usize> {
    let mut store = ZopfliLZ77Store::new(input);
    let mut s = ZopfliBlockState::new(options, instart, inend, false).unwrap();
    let mut hash = ZopfliHash::new(crate::util::ZOPFLI_WINDOW_SIZE);
    
    // Unintuitively, using a simple LZ77 method here instead of ZopfliLZ77Optimal
    // results in better blocks.
    crate::lz77::lz77_greedy(&mut s, input, instart, inend, &mut store, &mut hash);
    
    let lz77splitpoints = block_split_lz77(options, &store, maxblocks);
    
    // Convert LZ77 positions to positions in the uncompressed input.
    let mut splitpoints = Vec::new();
    let mut pos = instart;
    let mut npoints = 0;
    
    if !lz77splitpoints.is_empty() {
        for i in 0..store.size() {
            let length = if store.dists()[i] == 0 { 1 } else { store.litlens()[i] as usize };
            
            if npoints < lz77splitpoints.len() && lz77splitpoints[npoints] == i {
                splitpoints.push(pos);
                npoints += 1;
                if npoints == lz77splitpoints.len() {
                    break;
                }
            }
            pos += length;
        }
    }
    
    assert_eq!(npoints, lz77splitpoints.len());
    splitpoints
}

/// Divides the input into equal blocks, does not even take LZ77 lengths into
/// account.
pub fn block_split_simple(
    _in: &[u8], // The input parameter is marked (void)in in C, so content not used
    instart: usize,
    inend: usize,
    blocksize: usize
) -> Vec<usize> {
    let mut splitpoints = Vec::new();
    let mut i = instart;
    
    while i < inend {
        splitpoints.push(i);
        i += blocksize;
    }
    
    splitpoints
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_sorted() {
        let mut vec = Vec::new();
        add_sorted(5, &mut vec);
        add_sorted(2, &mut vec);
        add_sorted(8, &mut vec);
        add_sorted(3, &mut vec);
        
        assert_eq!(vec, vec![2, 3, 5, 8]);
    }
    
    #[test]
    fn test_find_minimum_simple() {
        let f = |x: usize| (x as f64 - 5.0).abs();
        let (pos, value) = find_minimum(f, 0, 10);
        assert_eq!(pos, 5);
        assert!((value - 0.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_block_split_simple() {
        let data = b"hello world test data";
        let splitpoints = block_split_simple(data, 0, data.len(), 5);
        
        assert_eq!(splitpoints, vec![0, 5, 10, 15, 20]);
    }
    
    #[test]
    fn test_find_largest_splittable_block() {
        // Create done array that matches our expected usage
        let done = vec![false, false, false, false, false, false, false, false, false, false, 
                       true, false, false, false, false, false, false, false, false, false,
                       false, false, false, false, false, false, false, false, false, false,
                       false, false, false, false, false, false, false, false, false, false];
        let splitpoints = vec![10, 20, 30];
        
        let result = find_largest_splittable_block(40, &done, &splitpoints);
        
        // Should find the largest block that's not done
        // Blocks are: [0, 10), [10, 20), [20, 30), [30, 39]
        // done[0] = false (block [0, 10) size 10)
        // done[10] = true (block [10, 20) size 10) - done
        // done[20] = false (block [20, 30) size 10)
        // done[30] = false (block [30, 39) size 9)
        
        assert!(result.is_some());
        let (start, _end) = result.unwrap();
        // Should pick one of the size-10 blocks that's not done
        assert!(start == 0 || start == 20);
    }
}