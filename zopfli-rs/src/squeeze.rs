use crate::lz77::{ZopfliBlockState, ZopfliLZ77Store};
use crate::hash::ZopfliHash;
use crate::tree::calculate_entropy;
use crate::symbols::{get_dist_extra_bits, get_dist_symbol, get_length_extra_bits, get_length_symbol};
use crate::util::{ZOPFLI_NUM_LL, ZOPFLI_NUM_D, ZOPFLI_LARGE_FLOAT, ZOPFLI_WINDOW_SIZE, ZOPFLI_MAX_MATCH, ZOPFLI_MIN_MATCH};

#[derive(Debug, Clone)]
pub struct SymbolStats {
    /// The literal and length symbols.
    pub litlens: [usize; ZOPFLI_NUM_LL],
    /// The 32 unique dist symbols, not the 32768 possible dists.
    pub dists: [usize; ZOPFLI_NUM_D],
    /// Length of each lit/len symbol in bits.
    pub ll_symbols: [f64; ZOPFLI_NUM_LL],
    /// Length of each dist symbol in bits.
    pub d_symbols: [f64; ZOPFLI_NUM_D],
}

impl Default for SymbolStats {
    fn default() -> Self {
        Self {
            litlens: [0; ZOPFLI_NUM_LL],
            dists: [0; ZOPFLI_NUM_D],
            ll_symbols: [0.0; ZOPFLI_NUM_LL],
            d_symbols: [0.0; ZOPFLI_NUM_D],
        }
    }
}

impl SymbolStats {
    /// Sets everything to 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear frequency counts
    pub fn clear_freqs(&mut self) {
        self.litlens.fill(0);
        self.dists.fill(0);
    }

    /// Adds the bit lengths.
    pub fn add_weighed_freqs(&mut self, stats1: &SymbolStats, w1: f64, stats2: &SymbolStats, w2: f64) {
        for i in 0..ZOPFLI_NUM_LL {
            self.litlens[i] = ((stats1.litlens[i] as f64) * w1 + (stats2.litlens[i] as f64) * w2) as usize;
        }
        for i in 0..ZOPFLI_NUM_D {
            self.dists[i] = ((stats1.dists[i] as f64) * w1 + (stats2.dists[i] as f64) * w2) as usize;
        }
        self.litlens[256] = 1; // End symbol.
    }

    /// Calculate the entropy of the statistics
    pub fn calculate_statistics(&mut self) {
        calculate_entropy(&self.litlens, &mut self.ll_symbols);
        calculate_entropy(&self.dists, &mut self.d_symbols);
    }

    /// Appends the symbol statistics from the store.
    pub fn get_statistics(&mut self, store: &ZopfliLZ77Store) {
        for i in 0..store.size() {
            let (litlen, dist) = store.get_litlen_dist(i);
            if dist == 0 {
                self.litlens[litlen as usize] += 1;
            } else {
                self.litlens[get_length_symbol(litlen as i32) as usize] += 1;
                self.dists[get_dist_symbol(dist as i32) as usize] += 1;
            }
        }
        self.litlens[256] = 1; // End symbol.
        self.calculate_statistics();
    }
}

#[derive(Debug)]
pub struct RanState {
    m_w: u32,
    m_z: u32,
}

impl RanState {
    pub fn new() -> Self {
        Self { m_w: 1, m_z: 2 }
    }

    /// Get random number: "Multiply-With-Carry" generator of G. Marsaglia
    pub fn ran(&mut self) -> u32 {
        self.m_z = 36969u32.wrapping_mul(self.m_z & 65535).wrapping_add(self.m_z >> 16);
        self.m_w = 18000u32.wrapping_mul(self.m_w & 65535).wrapping_add(self.m_w >> 16);
        (self.m_z << 16).wrapping_add(self.m_w) // 32-bit result.
    }

    pub fn randomize_freqs(&mut self, freqs: &mut [usize]) {
        let n = freqs.len();
        for i in 0..n {
            if (self.ran() >> 4) % 3 == 0 {
                freqs[i] = freqs[(self.ran() as usize) % n];
            }
        }
    }

    pub fn randomize_stat_freqs(&mut self, stats: &mut SymbolStats) {
        self.randomize_freqs(&mut stats.litlens);
        self.randomize_freqs(&mut stats.dists);
        stats.litlens[256] = 1; // End symbol.
    }
}

/// Function that calculates a cost based on a model for the given LZ77 symbol.
/// litlen: means literal symbol if dist is 0, length otherwise.
pub type CostModelFun = fn(litlen: u32, dist: u32, context: Option<&SymbolStats>) -> f64;

/// Cost model which should exactly match fixed tree.
pub fn get_cost_fixed(litlen: u32, dist: u32, _context: Option<&SymbolStats>) -> f64 {
    if dist == 0 {
        if litlen <= 143 {
            8.0
        } else {
            9.0
        }
    } else {
        let dbits = get_dist_extra_bits(dist as i32);
        let lbits = get_length_extra_bits(litlen as i32);
        let lsym = get_length_symbol(litlen as i32);
        let mut cost = 0;
        if lsym <= 279 {
            cost += 7;
        } else {
            cost += 8;
        }
        cost += 5; // Every dist symbol has length 5.
        (cost + dbits + lbits) as f64
    }
}

/// Cost model based on symbol statistics.
pub fn get_cost_stat(litlen: u32, dist: u32, context: Option<&SymbolStats>) -> f64 {
    let stats = context.expect("SymbolStats context required for get_cost_stat");
    
    if dist == 0 {
        stats.ll_symbols[litlen as usize]
    } else {
        let lsym = get_length_symbol(litlen as i32);
        let lbits = get_length_extra_bits(litlen as i32);
        let dsym = get_dist_symbol(dist as i32);
        let dbits = get_dist_extra_bits(dist as i32);
        (lbits + dbits) as f64 + stats.ll_symbols[lsym as usize] + stats.d_symbols[dsym as usize]
    }
}

/// Finds the minimum possible cost this cost model can return for valid length and distance symbols.
fn get_cost_model_min_cost(costmodel: CostModelFun, costcontext: Option<&SymbolStats>) -> f64 {
    // Table of distances that have a different distance symbol in the deflate
    // specification. Each value is the first distance that has a new symbol. Only
    // different symbols affect the cost model so only these need to be checked.
    // See RFC 1951 section 3.2.5. Compressed blocks (length and distance codes).
    const DSYMBOLS: [u32; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513,
        769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577
    ];

    let mut mincost = ZOPFLI_LARGE_FLOAT;
    let mut _bestlength = 0; // length that has lowest cost in the cost model
    for i in 3..259 {
        let c = costmodel(i, 1, costcontext);
        if c < mincost {
            _bestlength = i;
            mincost = c;
        }
    }

    let mut bestdist = 0; // distance that has lowest cost in the cost model
    mincost = ZOPFLI_LARGE_FLOAT;
    for i in 0..30 {
        let c = costmodel(3, DSYMBOLS[i], costcontext);
        if c < mincost {
            bestdist = DSYMBOLS[i];
            mincost = c;
        }
    }

    costmodel(_bestlength, bestdist, costcontext)
}

/// Performs the forward pass for "squeeze". Gets the most optimal length to reach
/// every byte from a previous byte, using cost calculations.
fn get_best_lengths(
    s: &mut ZopfliBlockState,
    input: &[u8],
    instart: usize,
    inend: usize,
    costmodel: CostModelFun,
    costcontext: Option<&SymbolStats>,
    length_array: &mut [u16],
    h: &mut ZopfliHash,
    costs: &mut [f32],
) -> f64 {
    let blocksize = inend - instart;
    let windowstart = if instart > ZOPFLI_WINDOW_SIZE {
        instart - ZOPFLI_WINDOW_SIZE
    } else {
        0
    };

    if instart == inend {
        return 0.0;
    }

    h.reset(ZOPFLI_WINDOW_SIZE);
    h.warmup(input, windowstart, inend);
    for i in windowstart..instart {
        h.update(input, i, inend);
    }

    for i in 1..=blocksize {
        costs[i] = ZOPFLI_LARGE_FLOAT as f32;
    }
    costs[0] = 0.0; // Because it's the start.
    length_array[0] = 0;

    let mincost = get_cost_model_min_cost(costmodel, costcontext);

    for i in instart..inend {
        let j = i - instart; // Index in the costs array and length_array.
        h.update(input, i, inend);

        #[cfg(feature = "zopfli-shortcut-long-repetitions")]
        {
            // If we're in a long repetition of the same character and have more than
            // ZOPFLI_MAX_MATCH characters before and after our position.
            if h.same[i & crate::util::ZOPFLI_WINDOW_MASK] > (ZOPFLI_MAX_MATCH * 2) as u16
                && i > instart + ZOPFLI_MAX_MATCH + 1
                && i + ZOPFLI_MAX_MATCH * 2 + 1 < inend
                && h.same[(i - ZOPFLI_MAX_MATCH) & crate::util::ZOPFLI_WINDOW_MASK] > ZOPFLI_MAX_MATCH as u16
            {
                let symbolcost = costmodel(ZOPFLI_MAX_MATCH as u32, 1, costcontext) as f32;
                // Set the length to reach each one to ZOPFLI_MAX_MATCH, and the cost to
                // the cost corresponding to that length. Doing this, we skip
                // ZOPFLI_MAX_MATCH values to avoid calling find_longest_match.
                for k in 0..ZOPFLI_MAX_MATCH {
                    costs[j + ZOPFLI_MAX_MATCH] = costs[j] + symbolcost;
                    length_array[j + ZOPFLI_MAX_MATCH] = ZOPFLI_MAX_MATCH as u16;
                    i += 1;
                    j += 1;
                    h.update(input, i, inend);
                }
            }
        }

        let mut sublen = vec![0u16; 259];
        let (_dist, leng) = s.find_longest_match(h, input, i, inend, ZOPFLI_MAX_MATCH, Some(&mut sublen));

        // Literal.
        if i + 1 <= inend {
            let new_cost = costmodel(input[i] as u32, 0, costcontext) as f32 + costs[j];
            assert!(new_cost >= 0.0);
            if new_cost < costs[j + 1] {
                costs[j + 1] = new_cost;
                length_array[j + 1] = 1;
            }
        }

        // Lengths.
        let kend = std::cmp::min(leng as usize, inend - i);
        let mincostaddcostj = mincost as f32 + costs[j];
        for k in 3..=kend {
            // Calling the cost model is expensive, avoid this if we are already at
            // the minimum possible cost that it can return.
            if costs[j + k] <= mincostaddcostj {
                continue;
            }

            let new_cost = costmodel(k as u32, sublen[k] as u32, costcontext) as f32 + costs[j];
            assert!(new_cost >= 0.0);
            if new_cost < costs[j + k] {
                assert!(k <= ZOPFLI_MAX_MATCH);
                costs[j + k] = new_cost;
                length_array[j + k] = k as u16;
            }
        }
    }

    assert!(costs[blocksize] >= 0.0);
    costs[blocksize] as f64
}

/// Calculates the optimal path of lz77 lengths to use, from the calculated
/// length_array. The length_array must contain the optimal length to reach that
/// byte. The path will be filled with the lengths to use, so its data size will be
/// the amount of lz77 symbols.
fn trace_backwards(size: usize, length_array: &[u16]) -> Vec<u16> {
    let mut path = Vec::new();
    if size == 0 {
        return path;
    }

    let mut index = size;
    loop {
        path.push(length_array[index]);
        assert!(length_array[index] as usize <= index);
        assert!(length_array[index] <= ZOPFLI_MAX_MATCH as u16);
        assert!(length_array[index] != 0);
        index -= length_array[index] as usize;
        if index == 0 {
            break;
        }
    }

    // Mirror result.
    path.reverse();
    path
}

fn follow_path(
    s: &mut ZopfliBlockState,
    input: &[u8],
    instart: usize,
    inend: usize,
    path: &[u16],
    store: &mut ZopfliLZ77Store,
    h: &mut ZopfliHash,
) {
    let windowstart = if instart > ZOPFLI_WINDOW_SIZE {
        instart - ZOPFLI_WINDOW_SIZE
    } else {
        0
    };

    if instart == inend {
        return;
    }

    h.reset(ZOPFLI_WINDOW_SIZE);
    h.warmup(input, windowstart, inend);
    for i in windowstart..instart {
        h.update(input, i, inend);
    }

    let mut pos = instart;
    for &length in path {
        assert!(pos < inend);

        h.update(input, pos, inend);

        // Add to output.
        if length >= ZOPFLI_MIN_MATCH as u16 {
            // Get the distance by recalculating longest match. The found length
            // should match the length from the path.
            let (dist, dummy_length) = s.find_longest_match(h, input, pos, inend, length as usize, None);
            debug_assert!(!(dummy_length != length && length > 2 && dummy_length > 2));
            crate::lz77::verify_len_dist(input, pos, dist, length);
            store.store_lit_len_dist(length, dist, pos);
        } else {
            let _length = 1;
            store.store_lit_len_dist(input[pos] as u16, 0, pos);
        }

        assert!(pos + length as usize <= inend);
        for j in 1..length as usize {
            h.update(input, pos + j, inend);
        }

        pos += length as usize;
    }
}

/// Does a single run for ZopfliLZ77Optimal. For good compression, repeated runs
/// with updated statistics should be performed.
fn lz77_optimal_run(
    s: &mut ZopfliBlockState,
    input: &[u8],
    instart: usize,
    inend: usize,
    length_array: &mut [u16],
    costmodel: CostModelFun,
    costcontext: Option<&SymbolStats>,
    store: &mut ZopfliLZ77Store,
    h: &mut ZopfliHash,
    costs: &mut [f32],
) -> f64 {
    let cost = get_best_lengths(s, input, instart, inend, costmodel, costcontext, length_array, h, costs);
    let path = trace_backwards(inend - instart, length_array);
    follow_path(s, input, instart, inend, &path, store, h);
    assert!(cost < ZOPFLI_LARGE_FLOAT);
    cost
}

/// Calculates lit/len and dist pairs for given data.
/// If instart is larger than 0, it uses values before instart as starting dictionary.
pub fn lz77_optimal<'a>(
    s: &mut ZopfliBlockState,
    input: &'a [u8],
    instart: usize,
    inend: usize,
    numiterations: i32,
    store: &mut ZopfliLZ77Store<'a>,
) {
    let blocksize = inend - instart;
    let mut length_array = vec![0u16; blocksize + 1];
    let mut currentstore = ZopfliLZ77Store::new(input);
    let mut hash = ZopfliHash::new(ZOPFLI_WINDOW_SIZE);
    let mut stats = SymbolStats::new();
    let mut beststats = SymbolStats::new();
    let mut laststats = SymbolStats::new();
    let mut costs = vec![0.0f32; blocksize + 1];
    let mut bestcost = ZOPFLI_LARGE_FLOAT;
    let mut lastcost = 0.0;
    // Try randomizing the costs a bit once the size stabilizes.
    let mut ran_state = RanState::new();
    let mut lastrandomstep = -1;

    // Do regular deflate, then loop multiple shortest path runs, each time using
    // the statistics of the previous run.

    // Initial run.
    crate::lz77::lz77_greedy(s, input, instart, inend, &mut currentstore, &mut hash);
    stats.get_statistics(&currentstore);

    // Repeat statistics with each time the cost model from the previous stat run.
    for i in 0..numiterations {
        currentstore.clear();
        lz77_optimal_run(
            s,
            input,
            instart,
            inend,
            &mut length_array,
            get_cost_stat,
            Some(&stats),
            &mut currentstore,
            &mut hash,
            &mut costs,
        );
        
        // Calculate block size for cost comparison (using dynamic tree, btype=2)
        let cost = calculate_block_size(&currentstore, 0, currentstore.size(), 2);
        
        if s.options().verbose_more != 0 || (s.options().verbose != 0 && cost < bestcost) {
            eprintln!("Iteration {}: {} bit", i, cost as i32);
        }
        
        if cost < bestcost {
            // Copy to the output store.
            *store = currentstore.clone();
            beststats = stats.clone();
            bestcost = cost;
        }
        
        laststats = stats.clone();
        stats.clear_freqs();
        stats.get_statistics(&currentstore);
        
        if lastrandomstep != -1 {
            // This makes it converge slower but better. Do it only once the
            // randomness kicks in so that if the user does few iterations, it gives a
            // better result sooner.
            let mut newstats = SymbolStats::new();
            newstats.add_weighed_freqs(&stats, 1.0, &laststats, 0.5);
            stats = newstats;
            stats.calculate_statistics();
        }
        
        if i > 5 && cost == lastcost {
            stats = beststats.clone();
            ran_state.randomize_stat_freqs(&mut stats);
            stats.calculate_statistics();
            lastrandomstep = i;
        }
        
        lastcost = cost;
    }
}

/// Does the same as lz77_optimal, but optimized for the fixed tree of the
/// deflate standard.
pub fn lz77_optimal_fixed<'a>(
    s: &mut ZopfliBlockState,
    input: &'a [u8],
    instart: usize,
    inend: usize,
    store: &mut ZopfliLZ77Store<'a>,
) {
    let blocksize = inend - instart;
    let mut length_array = vec![0u16; blocksize + 1];
    let mut hash = ZopfliHash::new(ZOPFLI_WINDOW_SIZE);
    let mut costs = vec![0.0f32; blocksize + 1];

    s.set_block_start(instart);
    s.set_block_end(inend);

    // Shortest path for fixed tree This one should give the shortest possible
    // result for fixed tree, no repeated runs are needed since the tree is known.
    lz77_optimal_run(
        s,
        input,
        instart,
        inend,
        &mut length_array,
        get_cost_fixed,
        None,
        store,
        &mut hash,
        &mut costs,
    );
}

// Calculate block size using C implementation for now
fn calculate_block_size(_lz77: &ZopfliLZ77Store, _lstart: usize, _lend: usize, _btype: i32) -> f64 {
    #[cfg(feature = "c-fallback")]
    {
        // We need to create a C-compatible LZ77Store
        // For now, return a dummy value. This will be properly implemented when we port deflate.c
        1000.0
    }
    
    #[cfg(not(feature = "c-fallback"))]
    {
        // For now, return a dummy value. This will be properly implemented when we port deflate.c
        1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_stats() {
        let mut stats = SymbolStats::new();
        assert_eq!(stats.litlens[0], 0);
        assert_eq!(stats.dists[0], 0);
        
        stats.litlens[65] = 10;
        stats.dists[5] = 20;
        stats.calculate_statistics();
        
        // Test clear_freqs
        stats.clear_freqs();
        assert_eq!(stats.litlens[65], 0);
        assert_eq!(stats.dists[5], 0);
    }

    #[test]
    fn test_ran_state() {
        let mut ran = RanState::new();
        let r1 = ran.ran();
        let r2 = ran.ran();
        assert_ne!(r1, r2); // Should generate different numbers
    }

    #[test]
    fn test_cost_models() {
        // Test fixed cost model
        let cost_lit = get_cost_fixed(65, 0, None);
        assert_eq!(cost_lit, 8.0); // Literal <= 143 should cost 8 bits
        
        let cost_lit_high = get_cost_fixed(200, 0, None);
        assert_eq!(cost_lit_high, 9.0); // Literal > 143 should cost 9 bits
        
        let cost_match = get_cost_fixed(10, 100, None);
        assert!(cost_match > 0.0); // Match should have positive cost
    }

    #[test]
    fn test_trace_backwards() {
        let length_array = vec![0, 1, 1, 3, 1, 1, 3];
        let path = trace_backwards(6, &length_array);
        // Starting at index 6 with length_array[6] = 3, then index 3 with length_array[3] = 3
        // This traces: 6 -> 3 -> 0, so path is [3, 3] reversed = [3, 3]
        assert_eq!(path, vec![3, 3]);
    }

    #[test]
    #[cfg(feature = "c-fallback")]
    fn test_squeeze_simple_comparison() {
        use crate::options::ZopfliOptions;
        use crate::lz77::{ZopfliBlockState, ZopfliLZ77Store};
        
        // Simple test data
        let data = b"hello world hello";
        let options = ZopfliOptions::default();
        
        // Rust implementation
        let mut rust_block_state = ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
        let mut rust_store = ZopfliLZ77Store::new(data);
        lz77_optimal_fixed(&mut rust_block_state, data, 0, data.len(), &mut rust_store);
        
        // C implementation
        let mut c_block_state: crate::ffi::ZopfliBlockStateC = unsafe { std::mem::zeroed() };
        unsafe {
            crate::ffi::ZopfliInitBlockState(
                &options as *const ZopfliOptions,
                0,
                data.len(),
                0,
                &mut c_block_state,
            );
        }

        let mut c_store: crate::ffi::ZopfliLZ77StoreC = unsafe { std::mem::zeroed() };
        unsafe {
            crate::ffi::ZopfliInitLZ77Store(data.as_ptr(), &mut c_store);
            
            crate::ffi::ZopfliLZ77OptimalFixed(
                &mut c_block_state,
                data.as_ptr(),
                0,
                data.len(),
                &mut c_store,
            );
        }

        // Compare results exactly
        let c_size = unsafe { crate::ffi::ZopfliLZ77StoreGetSize(&c_store) };
        
        println!("Rust store size: {}, C store size: {}", rust_store.size(), c_size);
        assert_eq!(rust_store.size(), c_size, "Store sizes differ: Rust={}, C={}", rust_store.size(), c_size);
        
        // Compare each LZ77 symbol
        for i in 0..rust_store.size() {
            let (rust_litlen, rust_dist) = rust_store.get_litlen_dist(i);
            let c_litlen = unsafe { crate::ffi::ZopfliLZ77StoreGetLitLen(&c_store, i) };
            let c_dist = unsafe { crate::ffi::ZopfliLZ77StoreGetDist(&c_store, i) };
            
            println!("Index {}: Rust=({}, {}), C=({}, {})", i, rust_litlen, rust_dist, c_litlen, c_dist);
            
            assert_eq!(rust_litlen, c_litlen, 
                "LitLen mismatch at index {}: Rust={}, C={}", i, rust_litlen, c_litlen);
            assert_eq!(rust_dist, c_dist,
                "Dist mismatch at index {}: Rust={}, C={}", i, rust_dist, c_dist);
        }
        
        // Clean up C resources
        unsafe {
            crate::ffi::ZopfliCleanLZ77Store(&mut c_store);
            crate::ffi::ZopfliCleanBlockState(&mut c_block_state);
        }
    }
}