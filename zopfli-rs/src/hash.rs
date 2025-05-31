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

// Constants from util.h
const ZOPFLI_WINDOW_SIZE: usize = 32768;
const ZOPFLI_WINDOW_MASK: usize = ZOPFLI_WINDOW_SIZE - 1;
const ZOPFLI_MIN_MATCH: usize = 3;

// Constants from hash.c
const HASH_SHIFT: i32 = 5;
const HASH_MASK: i32 = 32767;

/// Hash table for ZopfliFindLongestMatch of lz77.c.
/// 
/// This struct corresponds to the ZopfliHash struct in the C code,
/// with all the conditional compilation flags enabled as per CODEBASE_ANALYSIS.md:
/// - ZOPFLI_HASH_SAME
/// - ZOPFLI_HASH_SAME_HASH
pub struct ZopfliHash {
    /// Hash value to index of its most recent occurrence.
    head: Vec<i32>,
    /// Index to index of prev. occurrence of same hash.
    prev: Vec<u16>,
    /// Index to hash value at this index.
    hashval: Vec<i32>,
    /// Current hash value.
    val: i32,
    
    // ZOPFLI_HASH_SAME_HASH fields
    /// Hash value to index of its most recent occurrence (second hash).
    head2: Vec<i32>,
    /// Index to index of prev. occurrence of same hash (second hash).
    prev2: Vec<u16>,
    /// Index to hash value at this index (second hash).
    hashval2: Vec<i32>,
    /// Current hash value for second hash.
    val2: i32,
    
    // ZOPFLI_HASH_SAME fields
    /// Amount of repetitions of same byte after this position.
    same: Vec<u16>,
}

impl ZopfliHash {
    /// Creates a new ZopfliHash with the given window size.
    /// This combines ZopfliAllocHash and ZopfliResetHash functionality.
    pub fn new(window_size: usize) -> Self {
        let hash = ZopfliHash {
            head: vec![-1; 65536],
            prev: (0..window_size).map(|i| i as u16).collect(),
            hashval: vec![-1; window_size],
            val: 0,
            head2: vec![-1; 65536],
            prev2: (0..window_size).map(|i| i as u16).collect(),
            hashval2: vec![-1; window_size],
            val2: 0,
            same: vec![0; window_size],
        };
        
        hash
    }
    
    /// Resets all fields of ZopfliHash.
    pub fn reset(&mut self, window_size: usize) {
        self.val = 0;
        
        // Reset head arrays to -1
        for i in 0..65536 {
            self.head[i] = -1;
            self.head2[i] = -1;
        }
        
        // Reset window-sized arrays
        for i in 0..window_size {
            self.prev[i] = i as u16;  // If prev[j] == j, then prev[j] is uninitialized
            self.hashval[i] = -1;
            self.prev2[i] = i as u16;
            self.hashval2[i] = -1;
            self.same[i] = 0;
        }
        
        self.val2 = 0;
    }
    
    /// Updates the sliding hash value with the given byte.
    /// All calls to this function must be made on consecutive input characters.
    fn update_hash_value(&mut self, c: u8) {
        self.val = (((self.val) << HASH_SHIFT) ^ (c as i32)) & HASH_MASK;
    }
    
    /// Updates the hash values based on the current position in the array.
    /// All calls to this must be made for consecutive bytes.
    pub fn update(&mut self, array: &[u8], pos: usize, end: usize) {
        let hpos = (pos & ZOPFLI_WINDOW_MASK) as u16;
        
        // Update hash value
        let byte_to_hash = if pos + ZOPFLI_MIN_MATCH <= end {
            array[pos + ZOPFLI_MIN_MATCH - 1]
        } else {
            0
        };
        self.update_hash_value(byte_to_hash);
        
        self.hashval[hpos as usize] = self.val;
        
        // Update prev chain
        if self.head[self.val as usize] != -1 && 
           self.hashval[self.head[self.val as usize] as usize] == self.val {
            self.prev[hpos as usize] = self.head[self.val as usize] as u16;
        } else {
            self.prev[hpos as usize] = hpos;
        }
        self.head[self.val as usize] = hpos as i32;
        
        // Update "same" array (ZOPFLI_HASH_SAME)
        let mut amount = 0u16;
        if pos > 0 {
            let prev_same_idx = ((pos - 1) & ZOPFLI_WINDOW_MASK) as usize;
            if self.same[prev_same_idx] > 1 {
                amount = self.same[prev_same_idx] - 1;
            }
        }
        
        while pos + amount as usize + 1 < end &&
              array[pos] == array[pos + amount as usize + 1] && 
              amount < u16::MAX {
            amount += 1;
        }
        self.same[hpos as usize] = amount;
        
        // Update second hash (ZOPFLI_HASH_SAME_HASH)
        self.val2 = ((self.same[hpos as usize].wrapping_sub(ZOPFLI_MIN_MATCH as u16) & 255) as i32) ^ self.val;
        self.hashval2[hpos as usize] = self.val2;
        
        if self.head2[self.val2 as usize] != -1 && 
           self.hashval2[self.head2[self.val2 as usize] as usize] == self.val2 {
            self.prev2[hpos as usize] = self.head2[self.val2 as usize] as u16;
        } else {
            self.prev2[hpos as usize] = hpos;
        }
        self.head2[self.val2 as usize] = hpos as i32;
    }
    
    /// Prepopulates hash: Fills in the initial values in the hash, 
    /// before update() can be used correctly.
    pub fn warmup(&mut self, array: &[u8], pos: usize, end: usize) {
        if pos < array.len() {
            self.update_hash_value(array[pos]);
        }
        if pos + 1 < end && pos + 1 < array.len() {
            self.update_hash_value(array[pos + 1]);
        }
    }
    
    // Getters for accessing the hash data
    pub fn head(&self) -> &[i32] {
        &self.head
    }
    
    pub fn prev(&self) -> &[u16] {
        &self.prev
    }
    
    pub fn hashval(&self) -> &[i32] {
        &self.hashval
    }
    
    pub fn val(&self) -> i32 {
        self.val
    }
    
    pub fn head2(&self) -> &[i32] {
        &self.head2
    }
    
    pub fn prev2(&self) -> &[u16] {
        &self.prev2
    }
    
    pub fn hashval2(&self) -> &[i32] {
        &self.hashval2
    }
    
    pub fn val2(&self) -> i32 {
        self.val2
    }
    
    pub fn same(&self) -> &[u16] {
        &self.same
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_creation() {
        let hash = ZopfliHash::new(1024);
        
        // Check initial state
        assert_eq!(hash.val, 0);
        assert_eq!(hash.val2, 0);
        
        // Check head arrays are initialized to -1
        assert!(hash.head.iter().all(|&x| x == -1));
        assert!(hash.head2.iter().all(|&x| x == -1));
        
        // Check prev arrays are initialized to identity
        for i in 0..1024 {
            assert_eq!(hash.prev[i], i as u16);
            assert_eq!(hash.prev2[i], i as u16);
        }
        
        // Check hashval arrays are initialized to -1
        assert!(hash.hashval.iter().all(|&x| x == -1));
        assert!(hash.hashval2.iter().all(|&x| x == -1));
        
        // Check same array is initialized to 0
        assert!(hash.same.iter().all(|&x| x == 0));
    }
    
    #[test]
    fn test_hash_warmup() {
        let mut hash = ZopfliHash::new(1024);
        let data = b"hello world";
        
        hash.warmup(data, 0, data.len());
        
        // The hash value should be updated after warmup
        // This is based on the C implementation logic
        let expected_val = (((0 << HASH_SHIFT) ^ (b'h' as i32)) << HASH_SHIFT ^ (b'e' as i32)) & HASH_MASK;
        assert_eq!(hash.val, expected_val);
    }
    
    #[test]
    fn test_hash_update() {
        let mut hash = ZopfliHash::new(1024);
        let data = b"abcabc";
        
        // Warmup first
        hash.warmup(data, 0, data.len());
        
        // Update at position 0
        hash.update(data, 0, data.len());
        
        // Check that the hash value was stored
        let hpos = 0 & ZOPFLI_WINDOW_MASK;
        assert_eq!(hash.hashval[hpos], hash.val);
        
        // Check head was updated
        assert_eq!(hash.head[hash.val as usize], hpos as i32);
    }
    
    #[test]
    fn test_same_detection() {
        let mut hash = ZopfliHash::new(1024);
        let data = b"aaabbbccc";
        
        hash.warmup(data, 0, data.len());
        
        // Update at position 0 (first 'a')
        hash.update(data, 0, data.len());
        let hpos0 = 0 & ZOPFLI_WINDOW_MASK;
        // Should detect 2 more 'a's ahead
        assert_eq!(hash.same[hpos0], 2);
        
        // Update at position 1 (second 'a')  
        hash.update(data, 1, data.len());
        let hpos1 = 1 & ZOPFLI_WINDOW_MASK;
        // Should detect 1 more 'a' ahead
        assert_eq!(hash.same[hpos1], 1);
        
        // Update at position 2 (third 'a')
        hash.update(data, 2, data.len());
        let hpos2 = 2 & ZOPFLI_WINDOW_MASK;
        // Should detect 0 more 'a's ahead (next is 'b')
        assert_eq!(hash.same[hpos2], 0);
    }
}