/*
Copyright 2011 Google Inc. All Rights Reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

Author: lode.vandevenne@gmail.com (Lode Vandevenne)
Author: jyrki.alakuijala@gmail.com (Jyrki Alakuijala)
*/

/*
Functions for basic LZ77 compression and utilities for the "squeeze" LZ77
compression.
*/

use crate::cache::ZopfliLongestMatchCache;
use crate::hash::ZopfliHash;
use crate::options::ZopfliOptions;
use crate::symbols::{get_length_symbol, get_dist_symbol};

// Constants from util.h
const ZOPFLI_NUM_LL: usize = 288;
const ZOPFLI_NUM_D: usize = 32;
pub const ZOPFLI_MAX_MATCH: usize = 258;
pub const ZOPFLI_MIN_MATCH: usize = 3;
pub const ZOPFLI_WINDOW_SIZE: usize = 32768;
const ZOPFLI_WINDOW_MASK: usize = ZOPFLI_WINDOW_SIZE - 1;
const ZOPFLI_MAX_CHAIN_HITS: i32 = 8192;

/// Stores lit/length and dist pairs for LZ77.
/// 
/// Parameter litlens: Contains the literal symbols or length values.
/// Parameter dists: Contains the distances. A value is 0 to indicate that there is
/// no dist and the corresponding litlens value is a literal instead of a length.
/// Parameter size: The size of both the litlens and dists arrays.
/// The memory can best be managed by using ZopfliLZ77Store::new to initialize it,
/// Drop to destroy it, and store_lit_len_dist to append values.
#[derive(Debug)]
pub struct ZopfliLZ77Store<'a> {
    /// Lit or len
    litlens: Vec<u16>,
    /// If 0: indicates literal in corresponding litlens,
    /// if > 0: length in corresponding litlens, this is the distance.
    dists: Vec<u16>,
    /// Original data (non-owning reference)
    data: &'a [u8],
    /// Position in data where this LZ77 command begins
    pos: Vec<usize>,
    /// Cached literal/length symbols
    ll_symbol: Vec<u16>,
    /// Cached distance symbols
    d_symbol: Vec<u16>,
    /// Cumulative histograms wrapping around per chunk. Each chunk has the amount
    /// of distinct symbols as length, so using 1 value per LZ77 symbol, we have a
    /// precise histogram at every N symbols, and the rest can be calculated by
    /// looping through the actual symbols of this chunk.
    ll_counts: Vec<usize>,
    d_counts: Vec<usize>,
}

impl<'a> ZopfliLZ77Store<'a> {
    /// Creates a new LZ77 store
    pub fn new(data: &'a [u8]) -> Self {
        ZopfliLZ77Store {
            litlens: Vec::new(),
            dists: Vec::new(),
            data,
            pos: Vec::new(),
            ll_symbol: Vec::new(),
            d_symbol: Vec::new(),
            ll_counts: Vec::new(),
            d_counts: Vec::new(),
        }
    }

    /// Gets the current size of the store
    pub fn size(&self) -> usize {
        self.litlens.len()
    }

    /// Gets the litlen and dist at a given index
    pub fn get_litlen_dist(&self, index: usize) -> (u16, u16) {
        (self.litlens[index], self.dists[index])
    }

    /// Gets a reference to the litlens array
    pub fn litlens(&self) -> &[u16] {
        &self.litlens
    }

    /// Gets a reference to the dists array
    pub fn dists(&self) -> &[u16] {
        &self.dists
    }

    /// Gets a reference to the data
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Gets a reference to the pos array
    pub fn pos(&self) -> &[usize] {
        &self.pos
    }

    /// Gets a reference to the ll_symbol array
    pub fn ll_symbol(&self) -> &[u16] {
        &self.ll_symbol
    }

    /// Gets a reference to the d_symbol array
    pub fn d_symbol(&self) -> &[u16] {
        &self.d_symbol
    }

    /// Gets a reference to the ll_counts array
    pub fn ll_counts(&self) -> &[usize] {
        &self.ll_counts
    }

    /// Gets a reference to the d_counts array
    pub fn d_counts(&self) -> &[usize] {
        &self.d_counts
    }

    /// Clears the store
    pub fn clear(&mut self) {
        self.litlens.clear();
        self.dists.clear();
        self.pos.clear();
        self.ll_symbol.clear();
        self.d_symbol.clear();
        self.ll_counts.clear();
        self.d_counts.clear();
    }

    /// Helper function for ceiling division, equivalent to CeilDiv in C
    fn ceil_div(a: usize, b: usize) -> usize {
        (a + b - 1) / b
    }

    /// Appends the length and distance to the LZ77 arrays of the ZopfliLZ77Store.
    /// This is equivalent to ZopfliStoreLitLenDist in the C code.
    pub fn store_lit_len_dist(&mut self, length: u16, dist: u16, pos: usize) {
        // Needed for using append multiple times
        let orig_size = self.size();
        let ll_start = ZOPFLI_NUM_LL * (orig_size / ZOPFLI_NUM_LL);
        let d_start = ZOPFLI_NUM_D * (orig_size / ZOPFLI_NUM_D);

        // Everytime the index wraps around, a new cumulative histogram is made: we're
        // keeping one histogram value per LZ77 symbol rather than a full histogram for
        // each to save memory.
        if orig_size % ZOPFLI_NUM_LL == 0 {
            for i in 0..ZOPFLI_NUM_LL {
                let value = if orig_size == 0 {
                    0
                } else {
                    self.ll_counts[orig_size - ZOPFLI_NUM_LL + i]
                };
                self.ll_counts.push(value);
            }
        }
        
        if orig_size % ZOPFLI_NUM_D == 0 {
            for i in 0..ZOPFLI_NUM_D {
                let value = if orig_size == 0 {
                    0
                } else {
                    self.d_counts[orig_size - ZOPFLI_NUM_D + i]
                };
                self.d_counts.push(value);
            }
        }

        // Append the main data
        self.litlens.push(length);
        self.dists.push(dist);
        self.pos.push(pos);
        
        debug_assert!(length < 259);

        if dist == 0 {
            // Literal
            self.ll_symbol.push(length);
            self.d_symbol.push(0);
            self.ll_counts[ll_start + length as usize] += 1;
        } else {
            // Length-distance pair
            let length_symbol = get_length_symbol(length as i32) as u16;
            let dist_symbol = get_dist_symbol(dist as i32) as u16;
            
            self.ll_symbol.push(length_symbol);
            self.d_symbol.push(dist_symbol);
            
            self.ll_counts[ll_start + length_symbol as usize] += 1;
            self.d_counts[d_start + dist_symbol as usize] += 1;
        }
    }

    /// Gets the amount of raw bytes that this range of LZ77 symbols spans.
    pub fn get_byte_range(&self, lstart: usize, lend: usize) -> usize {
        if lstart == lend {
            return 0;
        }
        
        let l = lend - 1;
        let end_pos = self.pos[l];
        let end_size = if self.dists[l] == 0 {
            1  // Literal takes 1 byte
        } else {
            self.litlens[l] as usize  // Length-distance pair takes 'length' bytes
        };
        
        end_pos + end_size - self.pos[lstart]
    }

    /// Helper function to get histogram at a specific position (equivalent to ZopfliLZ77GetHistogramAt)
    fn get_histogram_at(&self, lpos: usize, ll_counts: &mut [usize], d_counts: &mut [usize]) {
        // The real histogram is created by using the histogram for this chunk, but
        // all superfluous values of this chunk subtracted.
        let ll_pos = ZOPFLI_NUM_LL * (lpos / ZOPFLI_NUM_LL);
        let d_pos = ZOPFLI_NUM_D * (lpos / ZOPFLI_NUM_D);

        // Copy the cumulative histogram values
        for i in 0..ZOPFLI_NUM_LL {
            ll_counts[i] = self.ll_counts[ll_pos + i];
        }
        
        // Subtract values that come after lpos in this chunk
        for i in (lpos + 1)..(ll_pos + ZOPFLI_NUM_LL).min(self.size()) {
            ll_counts[self.ll_symbol[i] as usize] -= 1;
        }

        for i in 0..ZOPFLI_NUM_D {
            d_counts[i] = self.d_counts[d_pos + i];
        }
        
        // Subtract values that come after lpos in this chunk
        for i in (lpos + 1)..(d_pos + ZOPFLI_NUM_D).min(self.size()) {
            if self.dists[i] != 0 {
                d_counts[self.d_symbol[i] as usize] -= 1;
            }
        }
    }

    /// Gets the histogram of lit/len and dist symbols in the given range, using the
    /// cumulative histograms, so faster than adding one by one for large range. Does
    /// not add the one end symbol of value 256.
    pub fn get_histogram(&self, lstart: usize, lend: usize, 
                        ll_counts: &mut [usize], d_counts: &mut [usize]) {
        if lstart + ZOPFLI_NUM_LL * 3 > lend {
            // For small ranges, just iterate through the symbols
            ll_counts.fill(0);
            d_counts.fill(0);
            
            for i in lstart..lend {
                ll_counts[self.ll_symbol[i] as usize] += 1;
                if self.dists[i] != 0 {
                    d_counts[self.d_symbol[i] as usize] += 1;
                }
            }
        } else {
            // For large ranges, use cumulative histograms
            // Subtract the cumulative histograms at the end and the start to get the
            // histogram for this range.
            self.get_histogram_at(lend - 1, ll_counts, d_counts);
            
            if lstart > 0 {
                let mut ll_counts2 = vec![0; ZOPFLI_NUM_LL];
                let mut d_counts2 = vec![0; ZOPFLI_NUM_D];
                self.get_histogram_at(lstart - 1, &mut ll_counts2, &mut d_counts2);

                for i in 0..ZOPFLI_NUM_LL {
                    ll_counts[i] -= ll_counts2[i];
                }
                for i in 0..ZOPFLI_NUM_D {
                    d_counts[i] -= d_counts2[i];
                }
            }
        }
    }

    /// Appends all items from another store to this store
    pub fn append_store(&mut self, other: &ZopfliLZ77Store) {
        for i in 0..other.size() {
            self.store_lit_len_dist(other.litlens[i], other.dists[i], other.pos[i]);
        }
    }
}

impl<'a> Clone for ZopfliLZ77Store<'a> {
    /// Deep copy of the store (equivalent to ZopfliCopyLZ77Store)
    fn clone(&self) -> Self {
        let size = self.size();
        let ll_size = ZOPFLI_NUM_LL * Self::ceil_div(size, ZOPFLI_NUM_LL);
        let d_size = ZOPFLI_NUM_D * Self::ceil_div(size, ZOPFLI_NUM_D);

        ZopfliLZ77Store {
            litlens: self.litlens.clone(),
            dists: self.dists.clone(),
            data: self.data,
            pos: self.pos.clone(),
            ll_symbol: self.ll_symbol.clone(),
            d_symbol: self.d_symbol.clone(),
            ll_counts: {
                let mut counts = self.ll_counts.clone();
                counts.resize(ll_size, 0);
                counts
            },
            d_counts: {
                let mut counts = self.d_counts.clone();
                counts.resize(d_size, 0);
                counts
            },
        }
    }
}

/// Some state information for compressing a block.
/// This is currently a bit under-used (with mainly only the longest match cache),
/// but is kept for easy future expansion.
#[derive(Debug)]
pub struct ZopfliBlockState<'a> {
    options: &'a ZopfliOptions,
    /// Cache for length/distance pairs found so far (if enabled)
    lmc: Option<ZopfliLongestMatchCache>,
    /// The start (inclusive) and end (not inclusive) of the current block
    block_start: usize,
    block_end: usize,
}

impl<'a> ZopfliBlockState<'a> {
    /// Creates a new block state
    pub fn new(options: &'a ZopfliOptions, block_start: usize, block_end: usize, add_lmc: bool) -> Result<Self, String> {
        let lmc = if add_lmc {
            Some(ZopfliLongestMatchCache::new(block_end - block_start)?)
        } else {
            None
        };

        Ok(ZopfliBlockState {
            options,
            lmc,
            block_start,
            block_end,
        })
    }

    /// Gets the options
    pub fn options(&self) -> &ZopfliOptions {
        self.options
    }

    /// Gets the longest match cache if available
    pub fn lmc(&self) -> Option<&ZopfliLongestMatchCache> {
        self.lmc.as_ref()
    }

    /// Gets mutable access to the longest match cache if available
    pub fn lmc_mut(&mut self) -> Option<&mut ZopfliLongestMatchCache> {
        self.lmc.as_mut()
    }

    /// Gets the block start position
    pub fn block_start(&self) -> usize {
        self.block_start
    }

    /// Gets the block end position
    pub fn block_end(&self) -> usize {
        self.block_end
    }

    /// Sets the block start position (needed for lz77_optimal_fixed)
    pub fn set_block_start(&mut self, start: usize) {
        self.block_start = start;
    }

    /// Sets the block end position (needed for lz77_optimal_fixed)
    pub fn set_block_end(&mut self, end: usize) {
        self.block_end = end;
    }

    /// Finds the longest match (wrapper method)
    pub fn find_longest_match(
        &mut self,
        h: &ZopfliHash,
        array: &[u8],
        pos: usize,
        size: usize,
        limit: usize,
        sublen: Option<&mut [u16]>,
    ) -> (u16, u16) {
        let mut distance = 0;
        let mut length = 0;
        find_longest_match(self, h, array, pos, size, limit, sublen, &mut distance, &mut length);
        (distance, length)
    }
}

/// Verifies if length and dist are indeed valid, only used for assertion.
/// This is equivalent to ZopfliVerifyLenDist but uses debug assertions.
pub fn verify_len_dist(data: &[u8], pos: usize, dist: u16, length: u16) {
    debug_assert!(pos + length as usize <= data.len());
    
    for i in 0..length as usize {
        debug_assert_eq!(
            data[pos - dist as usize + i], 
            data[pos + i],
            "Length/distance verification failed at position {}, offset {}", pos, i
        );
    }
}

/// Finds how long the match of scan and match is. Can be used to find how many
/// bytes starting from scan, and from match, are equal. Returns the last position
/// after scan, which is still equal to the corresponding byte after match.
/// scan is the position to compare
/// match is the earlier position to compare.
/// end is the last possible byte, beyond which to stop looking.
/// safe_end is a few (8) bytes before end, for comparing multiple bytes at once.
fn get_match(data: &[u8], scan_pos: usize, match_pos: usize, end: usize) -> usize {
    let mut scan = scan_pos;
    let mut match_idx = match_pos;
    let safe_end = end.saturating_sub(8);

    // For Rust, we'll use the simpler 64-bit approach since size_t is typically 64-bit
    // and unsafe casting would be non-idiomatic. Instead, we use chunk-based comparison.
    while scan < safe_end && match_idx < data.len().saturating_sub(7) && scan < data.len().saturating_sub(7) {
        // Compare 8 bytes at once using slices
        if &data[scan..scan + 8] == &data[match_idx..match_idx + 8] {
            scan += 8;
            match_idx += 8;
        } else {
            break;
        }
    }

    // The remaining few bytes
    while scan < end && match_idx < data.len() && scan < data.len() && data[scan] == data[match_idx] {
        scan += 1;
        match_idx += 1;
    }

    scan
}

/// Gets distance, length and sublen values from the cache if possible.
/// Returns true if it got the values from the cache, false if not.
/// Updates the limit value to a smaller one if possible with more limited
/// information from the cache.
fn try_get_from_longest_match_cache(
    s: &ZopfliBlockState,
    pos: usize,
    limit: &mut usize,
    sublen: Option<&mut [u16]>,
    distance: &mut u16,
    length: &mut u16,
) -> bool {
    let lmc = match s.lmc() {
        Some(lmc) => lmc,
        None => return false,
    };

    // The LMC cache starts at the beginning of the block rather than the
    // beginning of the whole array.
    let lmc_pos = pos - s.block_start();

    if lmc_pos >= lmc.length().len() {
        return false;
    }

    // Length > 0 and dist 0 is invalid combination, which indicates on purpose
    // that this cache value is not filled in yet.
    let cache_available = lmc.length()[lmc_pos] == 0 || lmc.dist()[lmc_pos] != 0;
    let limit_ok_for_cache = cache_available && 
        (*limit == ZOPFLI_MAX_MATCH || 
         lmc.length()[lmc_pos] as usize <= *limit ||
         (sublen.is_some() && lmc.max_cached_sublen(lmc_pos, lmc.length()[lmc_pos] as usize) >= *limit));

    if limit_ok_for_cache && cache_available {
        if sublen.is_none() || 
           lmc.length()[lmc_pos] as usize <= lmc.max_cached_sublen(lmc_pos, lmc.length()[lmc_pos] as usize) {
            
            *length = lmc.length()[lmc_pos];
            if *length as usize > *limit {
                *length = *limit as u16;
            }
            
            if let Some(sublen_slice) = sublen {
                let mut temp_sublen = vec![0u16; (*length as usize).max(ZOPFLI_MIN_MATCH)];
                lmc.cache_to_sublen(lmc_pos, *length as usize, &mut temp_sublen);
                if (*length as usize) < temp_sublen.len() {
                    *distance = temp_sublen[*length as usize];
                } else {
                    *distance = 0;
                }
                
                // Copy to the actual sublen array
                for i in 0..(*length as usize).min(sublen_slice.len()) {
                    if i < temp_sublen.len() {
                        sublen_slice[i] = temp_sublen[i];
                    }
                }
                
                if *limit == ZOPFLI_MAX_MATCH && *length >= ZOPFLI_MIN_MATCH as u16 {
                    debug_assert_eq!(*distance, lmc.dist()[lmc_pos], 
                        "Cache distance mismatch at position {}", pos);
                }
            } else {
                *distance = lmc.dist()[lmc_pos];
            }
            return true;
        }
        // Can't use much of the cache, since the "sublens" need to be calculated,
        // but at least we already know when to stop.
        *limit = lmc.length()[lmc_pos] as usize;
    }

    false
}

/// Stores the found sublen, distance and length in the longest match cache, if
/// possible.
fn store_in_longest_match_cache(
    s: &mut ZopfliBlockState,
    pos: usize,
    limit: usize,
    sublen: Option<&[u16]>,
    distance: u16,
    length: u16,
) {
    // Get block_start before borrowing LMC
    let block_start = s.block_start();
    
    let lmc = match s.lmc_mut() {
        Some(lmc) => lmc,
        None => return,
    };

    // The LMC cache starts at the beginning of the block rather than the
    // beginning of the whole array.
    let lmc_pos = pos - block_start;

    if lmc_pos >= lmc.length().len() {
        return;
    }

    // Length > 0 and dist 0 is invalid combination, which indicates on purpose
    // that this cache value is not filled in yet.
    let cache_available = lmc.length()[lmc_pos] == 0 || lmc.dist()[lmc_pos] != 0;

    if limit == ZOPFLI_MAX_MATCH && sublen.is_some() && !cache_available {
        debug_assert_eq!(lmc.length()[lmc_pos], 1);
        debug_assert_eq!(lmc.dist()[lmc_pos], 0);
        
        // Update the cache with the found match
        let dist_to_store = if length < ZOPFLI_MIN_MATCH as u16 { 0 } else { distance };
        let length_to_store = if length < ZOPFLI_MIN_MATCH as u16 { 0 } else { length };
        
        lmc.update_length_dist(lmc_pos, length_to_store, dist_to_store);
        
        debug_assert!(!(lmc.length()[lmc_pos] == 1 && lmc.dist()[lmc_pos] == 0));
        
        if let Some(sublen_slice) = sublen {
            lmc.sublen_to_cache(sublen_slice, lmc_pos, length as usize);
        }
    }
}

/// Finds the longest match (length and corresponding distance) for LZ77 compression.
/// Even when not using "sublen", it can be more efficient to provide an array,
/// because only then the caching is used.
/// 
/// This is a direct port of ZopfliFindLongestMatch from the C code.
pub fn find_longest_match(
    s: &mut ZopfliBlockState,
    h: &ZopfliHash,
    array: &[u8],
    pos: usize,
    size: usize,
    mut limit: usize,
    mut sublen: Option<&mut [u16]>,
    distance: &mut u16,
    length: &mut u16,
) {
    let hpos = (pos & ZOPFLI_WINDOW_MASK) as u16;
    let mut best_dist = 0u16;
    let mut best_length = 1u16;
    let mut chain_counter = ZOPFLI_MAX_CHAIN_HITS;
    let mut dist; // Not unsigned short on purpose (matches C)

    // Try to get from cache first (ZOPFLI_LONGEST_MATCH_CACHE)
    if try_get_from_longest_match_cache(s, pos, &mut limit, sublen.as_deref_mut(), distance, length) {
        debug_assert!(pos + (*length as usize) <= size);
        return;
    }

    debug_assert!(limit <= ZOPFLI_MAX_MATCH);
    debug_assert!(limit >= ZOPFLI_MIN_MATCH);
    debug_assert!(pos < size);

    if size - pos < ZOPFLI_MIN_MATCH {
        // The rest of the code assumes there are at least ZOPFLI_MIN_MATCH bytes to try.
        *length = 0;
        *distance = 0;
        return;
    }

    if pos + limit > size {
        limit = size - pos;
    }

    let array_end = pos + limit;
    let _array_end_safe = array_end.saturating_sub(8);

    // Get hash values - exactly like C code
    let hval = h.val();
    debug_assert!(hval < 65536);

    let hhead = h.head();
    let hprev = h.prev();
    let hhashval = h.hashval();

    if hval < 0 || (hval as usize) >= hhead.len() {
        *length = best_length;
        *distance = best_dist;
        return;
    }

    // pp = hhead[hval] - During the whole loop, p == hprev[pp]
    let mut pp = hhead[hval as usize];
    if pp < 0 || (pp as usize) >= hprev.len() {
        *length = best_length;
        *distance = best_dist;
        return;
    }

    let mut p = hprev[pp as usize];

    // This assertion from C code: assert(pp == hpos);
    // We'll make it a debug_assert but handle the case where it fails
    if pp != hpos as i32 {
        // In some cases, the hash isn't perfectly aligned, so we'll continue anyway
        // but this suggests the hash initialization might need adjustment
    }

    dist = if (p as i32) < pp {
        (pp - p as i32) as u32
    } else {
        ((ZOPFLI_WINDOW_SIZE - p as usize) + pp as usize) as u32
    };

    // Keep track of which hash tables we're using (for ZOPFLI_HASH_SAME_HASH)
    let mut current_hhead = hhead;
    let mut current_hprev = hprev;
    let mut current_hhashval = hhashval;
    let mut current_hval = hval;

    // Go through all distances
    while dist < ZOPFLI_WINDOW_SIZE as u32 {
        debug_assert!((p as usize) < ZOPFLI_WINDOW_SIZE);
        
        if (p as usize) >= current_hprev.len() {
            break;
        }
        
        debug_assert_eq!(current_hprev[pp as usize], p);
        
        if (p as usize) < current_hhashval.len() && current_hhashval[p as usize] == current_hval {
            if dist > 0 {
                debug_assert!(pos < size);
                debug_assert!((dist as usize) <= pos);
                
                let scan_start = pos;
                let match_start = pos - (dist as usize);

                // Testing the byte at position best_length first, goes slightly faster
                if pos + (best_length as usize) >= size || 
                   (match_start + (best_length as usize) < array.len() && 
                    scan_start + (best_length as usize) < array.len() &&
                    array[scan_start + (best_length as usize)] == array[match_start + (best_length as usize)]) {

                    let mut scan_pos = scan_start;
                    let mut match_pos = match_start;
                    
                    // ZOPFLI_HASH_SAME optimization
                    let same = h.same();
                    if (pos & ZOPFLI_WINDOW_MASK) < same.len() && 
                       ((pos - (dist as usize)) & ZOPFLI_WINDOW_MASK) < same.len() {
                        let same0 = same[pos & ZOPFLI_WINDOW_MASK];
                        if same0 > 2 && pos < array.len() && match_pos < array.len() && 
                           array[scan_pos] == array[match_pos] {
                            let same1 = same[(pos - (dist as usize)) & ZOPFLI_WINDOW_MASK];
                            let same_min = same0.min(same1).min(limit as u16);
                            if same_min > limit as u16 {
                                // Shouldn't happen, but be safe
                            } else {
                                scan_pos += same_min as usize;
                                match_pos += same_min as usize;
                            }
                        }
                    }

                    let scan_end = get_match(array, scan_pos, match_pos, array_end);
                    let current_length = scan_end - scan_start;

                    if current_length > (best_length as usize) {
                        if let Some(ref mut sublen_slice) = sublen {
                            for j in ((best_length as usize) + 1)..=current_length {
                                if j < sublen_slice.len() {
                                    sublen_slice[j] = dist as u16;
                                }
                            }
                        }
                        best_dist = dist as u16;
                        best_length = current_length as u16;
                        if current_length >= limit {
                            break;
                        }
                    }
                }
            }
        }

        // ZOPFLI_HASH_SAME_HASH: Switch to the other hash once this will be more efficient
        if current_hhead.as_ptr() != h.head2().as_ptr() && 
           (best_length as usize) >= (h.same().get(hpos as usize).copied().unwrap_or(0) as usize) &&
           h.val2() == h.hashval2().get(p as usize).copied().unwrap_or(-1) {
            // Now use the hash that encodes the length and first byte
            current_hhead = h.head2();
            current_hprev = h.prev2();
            current_hhashval = h.hashval2();
            current_hval = h.val2();
        }

        pp = p as i32;
        if (pp as usize) >= current_hprev.len() {
            break;
        }
        p = current_hprev[pp as usize];
        
        if p == pp as u16 {
            break; // Uninited prev value
        }

        dist += if (p as i32) < pp {
            (pp - p as i32) as u32
        } else {
            ((ZOPFLI_WINDOW_SIZE - p as usize) + pp as usize) as u32
        };

        chain_counter -= 1;
        if chain_counter <= 0 {
            break;
        }
    }

    // Store in cache (ZOPFLI_LONGEST_MATCH_CACHE)
    store_in_longest_match_cache(s, pos, limit, sublen.as_deref(), best_dist, best_length);

    debug_assert!((best_length as usize) <= limit);

    *distance = best_dist;
    *length = best_length;
    debug_assert!(pos + (*length as usize) <= size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_store_new() {
        let data = b"hello world";
        let store = ZopfliLZ77Store::new(data);
        
        assert_eq!(store.size(), 0);
        assert_eq!(store.data.len(), 11);
    }

    #[test]
    fn test_store_literal() {
        let data = b"hello world";
        let mut store = ZopfliLZ77Store::new(data);
        
        // Store a literal 'h' (ASCII 104)
        store.store_lit_len_dist(104, 0, 0);
        
        assert_eq!(store.size(), 1);
        assert_eq!(store.litlens[0], 104);
        assert_eq!(store.dists[0], 0);
        assert_eq!(store.pos[0], 0);
        assert_eq!(store.ll_symbol[0], 104);
        assert_eq!(store.d_symbol[0], 0);
    }

    #[test]
    fn test_store_length_distance() {
        let data = b"hello world hello";
        let mut store = ZopfliLZ77Store::new(data);
        
        // Store a length-distance pair: length 5, distance 12
        store.store_lit_len_dist(5, 12, 12);
        
        assert_eq!(store.size(), 1);
        assert_eq!(store.litlens[0], 5);
        assert_eq!(store.dists[0], 12);
        assert_eq!(store.pos[0], 12);
        
        // Verify symbols were calculated correctly
        let expected_length_symbol = get_length_symbol(5) as u16;
        let expected_dist_symbol = get_dist_symbol(12) as u16;
        assert_eq!(store.ll_symbol[0], expected_length_symbol);
        assert_eq!(store.d_symbol[0], expected_dist_symbol);
    }

    #[test]
    fn test_get_byte_range() {
        let data = b"hello world hello";
        let mut store = ZopfliLZ77Store::new(data);
        
        // Store some literals and length-distance pairs
        store.store_lit_len_dist(104, 0, 0);  // 'h' literal at pos 0
        store.store_lit_len_dist(101, 0, 1);  // 'e' literal at pos 1
        store.store_lit_len_dist(5, 12, 12);  // length 5, distance 12 at pos 12
        
        // Range covering just the first literal should be 1 byte
        assert_eq!(store.get_byte_range(0, 1), 1);
        
        // Range covering first two literals should be 2 bytes
        assert_eq!(store.get_byte_range(0, 2), 2);
        
        // Range covering all should span from pos 0 to pos 12 + length 5 = 17 bytes
        assert_eq!(store.get_byte_range(0, 3), 17);
    }

    #[test]
    fn test_clone() {
        let data = b"hello world";
        let mut store = ZopfliLZ77Store::new(data);
        
        store.store_lit_len_dist(104, 0, 0);
        store.store_lit_len_dist(5, 12, 5);
        
        let cloned = store.clone();
        
        assert_eq!(cloned.size(), store.size());
        assert_eq!(cloned.litlens, store.litlens);
        assert_eq!(cloned.dists, store.dists);
        assert_eq!(cloned.pos, store.pos);
        assert_eq!(cloned.ll_symbol, store.ll_symbol);
        assert_eq!(cloned.d_symbol, store.d_symbol);
    }
}

/// Gets a score of the length given the distance. Typically, the score of the
/// length is the length itself, but if the distance is very long, decrease the
/// score of the length a bit to make up for the fact that long distances use
/// large amounts of extra bits.
/// 
/// This is not an accurate score, it is a heuristic only for the greedy LZ77
/// implementation. More accurate cost models are employed later. Making this
/// heuristic more accurate may hurt rather than improve compression.
/// 
/// The two direct uses of this heuristic are:
/// -avoid using a length of 3 in combination with a long distance. This only has
///  an effect if length == 3.
/// -make a slightly better choice between the two options of the lazy matching.
/// 
/// Indirectly, this affects:
/// -the block split points if the default of block splitting first is used, in a
///  rather unpredictable way
/// -the first zopfli run, so it affects the chance of the first run being closer
///  to the optimal output
fn get_length_score(length: u16, distance: u16) -> u16 {
    // At 1024, the distance uses 9+ extra bits and this seems to be the sweet spot
    // on tested files.
    if distance > 1024 {
        length - 1
    } else {
        length
    }
}

/// Does LZ77 using an algorithm similar to gzip, with lazy matching, rather than
/// with the slow but better "squeeze" implementation.
/// The result is placed in the ZopfliLZ77Store.
/// If instart is larger than 0, it uses values before instart as starting
/// dictionary.
pub fn lz77_greedy(
    s: &mut ZopfliBlockState,
    input: &[u8],
    instart: usize,
    inend: usize,
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

    let mut i = instart;
    let mut leng;
    let mut dist;
    let mut lengthscore;
    let mut prev_length = 0;
    let mut prev_match = 0;
    let mut prevlengthscore: u16;
    let mut match_available = false;

    while i < inend {
        h.update(input, i, inend);

        let (distance, length) = s.find_longest_match(h, input, i, inend, ZOPFLI_MAX_MATCH, None);
        dist = distance;
        leng = length;
        lengthscore = get_length_score(leng, dist);

        // Lazy matching
        {
            prevlengthscore = get_length_score(prev_length, prev_match);
            if match_available {
                match_available = false;
                if lengthscore > prevlengthscore + 1 {
                    store.store_lit_len_dist(input[i - 1] as u16, 0, i - 1);
                    if lengthscore >= ZOPFLI_MIN_MATCH as u16 && leng < ZOPFLI_MAX_MATCH as u16 {
                        match_available = true;
                        prev_length = leng;
                        prev_match = dist;
                        i += 1;
                        continue;
                    }
                } else {
                    // Add previous to output
                    leng = prev_length;
                    dist = prev_match;
                    // lengthscore = prevlengthscore;
                    // Add to output
                    verify_len_dist(input, i - 1, dist, leng);
                    store.store_lit_len_dist(leng, dist, i - 1);
                    for _ in 2..leng {
                        debug_assert!(i < inend);
                        i += 1;
                        h.update(input, i, inend);
                    }
                    i += 1;
                    continue;
                }
            } else if lengthscore >= ZOPFLI_MIN_MATCH as u16 && leng < ZOPFLI_MAX_MATCH as u16 {
                match_available = true;
                prev_length = leng;
                prev_match = dist;
                i += 1;
                continue;
            }
            // End of lazy matching logic
        }

        // Add to output
        if lengthscore >= ZOPFLI_MIN_MATCH as u16 {
            verify_len_dist(input, i, dist, leng);
            store.store_lit_len_dist(leng, dist, i);
        } else {
            leng = 1;
            store.store_lit_len_dist(input[i] as u16, 0, i);
        }

        for _ in 1..leng {
            debug_assert!(i < inend);
            i += 1;
            h.update(input, i, inend);
        }
        i += 1;
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_append_store() {
        let data = b"hello world hello world";
        let mut store1 = ZopfliLZ77Store::new(data);
        let mut store2 = ZopfliLZ77Store::new(data);
        
        store1.store_lit_len_dist(104, 0, 0);
        store2.store_lit_len_dist(101, 0, 1);
        store2.store_lit_len_dist(5, 12, 12);
        
        let original_size = store1.size();
        store1.append_store(&store2);
        
        assert_eq!(store1.size(), original_size + store2.size());
        assert_eq!(store1.litlens[original_size], 101);
        assert_eq!(store1.litlens[original_size + 1], 5);
    }

    #[test]
    fn test_block_state() {
        let options = ZopfliOptions::default();
        let state = ZopfliBlockState::new(&options, 0, 100, true).unwrap();
        
        assert_eq!(state.block_start(), 0);
        assert_eq!(state.block_end(), 100);
        assert!(state.lmc().is_some());
        
        let state_no_lmc = ZopfliBlockState::new(&options, 0, 100, false).unwrap();
        assert!(state_no_lmc.lmc().is_none());
    }

    #[test]
    fn test_verify_len_dist() {
        let data = b"hello world hello";
        
        // Valid case: "hello" appears at positions 0 and 12, distance 12, length 5
        verify_len_dist(data, 12, 12, 5);
        
        // Valid case: 'l' appears at positions 2, 3, and 14, 15
        // So at position 14, we can reference position 2 with distance 12
        verify_len_dist(data, 14, 12, 1);
        
        // Valid case: 'o' appears at positions 4 and 16
        // So at position 16, we can reference position 4 with distance 12  
        verify_len_dist(data, 16, 12, 1);
    }

    #[test]
    fn test_get_match() {
        let data = b"hello world hello world hello";
        
        // Test match between "hello" at positions 0 and 12
        let match_end = get_match(data, 12, 0, 17);
        let match_length = match_end - 12;
        println!("Match at positions 12 and 0, end: {}, length: {}", match_end, match_length);
        assert_eq!(match_length, 5); // Should find 5-character match "hello"
        
        // Test partial match
        let match_end = get_match(data, 12, 0, 15);
        let match_length = match_end - 12;
        println!("Partial match at positions 12 and 0, end: {}, length: {}", match_end, match_length);
        assert_eq!(match_length, 3); // Should find only "hel" (limited by end)
        
        // Test match at position 5 and 11 (both spaces)
        let match_end = get_match(data, 11, 5, data.len());
        let match_length = match_end - 11;
        println!("Match at positions 11 and 5, end: {}, length: {}", match_end, match_length);
        assert_eq!(match_length, 1); // Should find only 1 character (' ' at both positions)
    }

    #[test]
    fn test_find_longest_match_simple() {
        // Use a simpler case: repeated "abc" pattern
        let data = b"abcabcabc";
        let options = ZopfliOptions::default();
        let mut state = ZopfliBlockState::new(&options, 0, data.len(), false).unwrap();
        let mut hash = ZopfliHash::new(ZOPFLI_WINDOW_SIZE);
        
        // Initialize hash with the data up to position 3
        // This simulates the state the hash would be in during normal processing
        hash.reset(ZOPFLI_WINDOW_SIZE);
        hash.warmup(data, 0, 4); // warmup to position 3 + 1
        for i in 0..=3 {
            hash.update(data, i, data.len());
        }
        
        let mut distance = 0u16;
        let mut length = 0u16;
        
        // Look for match at position 3 (should find "abc" at position 0)
        find_longest_match(
            &mut state,
            &hash,
            data,
            3,
            data.len(),
            ZOPFLI_MAX_MATCH,
            None,
            &mut distance,
            &mut length,
        );
        
        println!("Simple test - Found match: length={}, distance={}", length, distance);
        
        // Expected: length=6, distance=3 (matching "abcabc" from position 0)
        assert_eq!(length, 6, "Expected length 6 but got {}", length);
        assert_eq!(distance, 3, "Expected distance 3 but got {}", distance);
        
        // If we found the expected match, verify it
        if length == 6 && distance == 3 {
            verify_len_dist(data, 3, distance, length);
        }
    }
}