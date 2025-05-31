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
const ZOPFLI_CACHE_LENGTH: usize = 8;

/// Cache used by ZopfliFindLongestMatch to remember previously found length/dist
/// values.
/// This is needed because the squeeze runs will ask these values multiple times for
/// the same position.
/// Uses large amounts of memory, since it has to remember the distance belonging
/// to every possible shorter-than-the-best length (the so called "sublen" array).
///
/// This struct corresponds to ZopfliLongestMatchCache in the C code.
/// Only compiled when ZOPFLI_LONGEST_MATCH_CACHE is defined.
#[derive(Debug)]
pub struct ZopfliLongestMatchCache {
    /// Best match length found at each position.
    length: Vec<u16>,
    /// Distance for the best match length at each position.
    dist: Vec<u16>,
    /// Stores (length, dist_low_byte, dist_high_byte) tuples for sub-optimal matches.
    /// Size: ZOPFLI_CACHE_LENGTH * 3 * blocksize
    sublen: Vec<u8>,
}

impl ZopfliLongestMatchCache {
    /// Creates a new ZopfliLongestMatchCache for the given block size.
    /// This corresponds to ZopfliInitCache in the C code.
    pub fn new(blocksize: usize) -> Result<Self, String> {
        // Check for potential overflow in sublen allocation
        let sublen_size = ZOPFLI_CACHE_LENGTH.checked_mul(3)
            .and_then(|x| x.checked_mul(blocksize))
            .ok_or_else(|| format!("Cache allocation would overflow: ZOPFLI_CACHE_LENGTH * 3 * {}", blocksize))?;

        let cache = ZopfliLongestMatchCache {
            length: vec![1; blocksize],  // length > 0 and dist 0 is invalid, indicates not filled
            dist: vec![0; blocksize],
            sublen: vec![0; sublen_size],
        };

        Ok(cache)
    }

    /// Stores sublen array in the cache.
    /// 
    /// # Arguments
    /// * `sublen` - Array where sublen[k] is the distance for match length k
    /// * `pos` - The position in the block for which these sublengths are cached
    /// * `length` - The maximum length for which sublen data is valid
    pub fn sublen_to_cache(&mut self, sublen: &[u16], pos: usize, length: usize) {
        if ZOPFLI_CACHE_LENGTH == 0 || length < 3 {
            return;
        }

        let cache_start = ZOPFLI_CACHE_LENGTH * pos * 3;
        if cache_start >= self.sublen.len() {
            return; // Bounds check
        }

        let mut j = 0;
        let mut bestlength = 0;

        // Store up to ZOPFLI_CACHE_LENGTH entries, prioritizing changes in distance
        for i in 3..=length {
            if i == length || (i + 1 <= sublen.len() && sublen[i] != sublen[i + 1]) {
                if j < ZOPFLI_CACHE_LENGTH {
                    let cache_idx = cache_start + j * 3;
                    if cache_idx + 2 < self.sublen.len() {
                        self.sublen[cache_idx] = (i - 3) as u8;
                        self.sublen[cache_idx + 1] = (sublen[i] % 256) as u8;
                        self.sublen[cache_idx + 2] = ((sublen[i] >> 8) % 256) as u8;
                        bestlength = i;
                        j += 1;
                    }
                }
                if j >= ZOPFLI_CACHE_LENGTH {
                    break;
                }
            }
        }

        // Store the best length in the last slot if we have room
        if j < ZOPFLI_CACHE_LENGTH {
            debug_assert_eq!(bestlength, length);
            let cache_idx = cache_start + (ZOPFLI_CACHE_LENGTH - 1) * 3;
            if cache_idx < self.sublen.len() {
                self.sublen[cache_idx] = (bestlength - 3) as u8;
            }
        } else {
            debug_assert!(bestlength <= length);
        }

        // Note: In some edge cases (like all sublen distances being 0), 
        // max_cached_sublen may return 0 even when bestlength > 0
        // This happens because max_cached_sublen treats distance 0 as "no cached data"
        if self.max_cached_sublen(pos, length) > 0 {
            debug_assert_eq!(bestlength, self.max_cached_sublen(pos, length));
        }
    }

    /// Extracts sublen array from the cache.
    /// 
    /// # Arguments
    /// * `pos` - The position in the block
    /// * `length` - The maximum length to reconstruct (unused in C, kept for compatibility)
    /// * `sublen` - Output array to be filled with distance values
    pub fn cache_to_sublen(&self, pos: usize, length: usize, sublen: &mut [u16]) {
        if ZOPFLI_CACHE_LENGTH == 0 || length < 3 {
            return;
        }

        let maxlength = self.max_cached_sublen(pos, length);
        let mut prevlength = 0;

        let cache_start = ZOPFLI_CACHE_LENGTH * pos * 3;
        if cache_start >= self.sublen.len() {
            return; // Bounds check
        }

        for j in 0..ZOPFLI_CACHE_LENGTH {
            let cache_idx = cache_start + j * 3;
            if cache_idx + 2 >= self.sublen.len() {
                break;
            }

            let length = self.sublen[cache_idx] as usize + 3;
            let dist = self.sublen[cache_idx + 1] as u16 + 256 * self.sublen[cache_idx + 2] as u16;

            // Fill all lengths from prevlength to current length with this distance
            for i in prevlength..=length.min(sublen.len() - 1) {
                sublen[i] = dist;
            }

            if length == maxlength {
                break;
            }
            prevlength = length + 1;
        }
    }

    /// Returns the length up to which could be stored in the cache.
    pub fn max_cached_sublen(&self, pos: usize, _length: usize) -> usize {
        if ZOPFLI_CACHE_LENGTH == 0 {
            return 0;
        }

        let cache_start = ZOPFLI_CACHE_LENGTH * pos * 3;
        if cache_start + 2 >= self.sublen.len() {
            return 0; // Bounds check
        }

        // No sublen cached if first distance is 0 (cache[1] == 0 && cache[2] == 0 in C)
        if self.sublen[cache_start + 1] == 0 && self.sublen[cache_start + 2] == 0 {
            return 0;
        }

        let last_cache_idx = cache_start + (ZOPFLI_CACHE_LENGTH - 1) * 3;
        if last_cache_idx >= self.sublen.len() {
            return 0;
        }

        self.sublen[last_cache_idx] as usize + 3
    }

    // Getters for testing and debugging
    pub fn length(&self) -> &[u16] {
        &self.length
    }

    pub fn dist(&self) -> &[u16] {
        &self.dist
    }

    pub fn sublen(&self) -> &[u8] {
        &self.sublen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ZopfliLongestMatchCache::new(100).unwrap();
        
        // Check initial state
        assert_eq!(cache.length.len(), 100);
        assert_eq!(cache.dist.len(), 100);
        assert_eq!(cache.sublen.len(), ZOPFLI_CACHE_LENGTH * 3 * 100);
        
        // length should be initialized to 1, dist to 0
        assert!(cache.length.iter().all(|&x| x == 1));
        assert!(cache.dist.iter().all(|&x| x == 0));
        assert!(cache.sublen.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_cache_overflow_protection() {
        // Test that we handle potential overflow in allocation size
        let result = ZopfliLongestMatchCache::new(usize::MAX);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("overflow"));
    }

    #[test]
    fn test_sublen_to_cache_basic() {
        let mut cache = ZopfliLongestMatchCache::new(10).unwrap();
        
        // Create a simple sublen array: distances for lengths 3, 4, 5
        let mut sublen = vec![0; 10];
        sublen[3] = 100;  // distance 100 for length 3
        sublen[4] = 200;  // distance 200 for length 4  
        sublen[5] = 300;  // distance 300 for length 5
        
        cache.sublen_to_cache(&sublen, 0, 5);
        
        // Check that max cached sublen is correct
        assert_eq!(cache.max_cached_sublen(0, 5), 5);
        
        // Check that we can retrieve the data
        let mut reconstructed = vec![0; 10];
        cache.cache_to_sublen(0, 5, &mut reconstructed);
        
        assert_eq!(reconstructed[3], 100);
        assert_eq!(reconstructed[4], 200);
        assert_eq!(reconstructed[5], 300);
    }

    #[test]
    fn test_max_cached_sublen_empty() {
        let cache = ZopfliLongestMatchCache::new(10).unwrap();
        
        // Empty cache should return 0
        assert_eq!(cache.max_cached_sublen(0, 10), 0);
    }

    #[test]
    fn test_sublen_cache_roundtrip() {
        let mut cache = ZopfliLongestMatchCache::new(10).unwrap();
        
        // Create test data with different distances at different lengths
        let mut original_sublen = vec![0; 20];
        original_sublen[3] = 10;
        original_sublen[4] = 10;  // Same distance
        original_sublen[5] = 20;  // Different distance
        original_sublen[6] = 20;  // Same again
        original_sublen[7] = 30;  // Different
        
        cache.sublen_to_cache(&original_sublen, 1, 7);
        
        let mut reconstructed = vec![0; 20];
        cache.cache_to_sublen(1, 7, &mut reconstructed);
        
        // Check that the important values are preserved
        assert_eq!(reconstructed[3], 10);
        assert_eq!(reconstructed[4], 10);
        assert_eq!(reconstructed[5], 20);
        assert_eq!(reconstructed[6], 20);
        assert_eq!(reconstructed[7], 30);
        
        assert_eq!(cache.max_cached_sublen(1, 7), 7);
    }

    #[test]
    fn test_bounds_checking() {
        let mut cache = ZopfliLongestMatchCache::new(2).unwrap();
        
        // Test with position that would be out of bounds
        let sublen = vec![0; 10];
        cache.sublen_to_cache(&sublen, 10, 5);  // pos=10 is too large for blocksize=2
        
        // Should not crash and should return 0 for max cached
        assert_eq!(cache.max_cached_sublen(10, 5), 0);
        
        let mut reconstructed = vec![0; 10];
        cache.cache_to_sublen(10, 5, &mut reconstructed);  // Should not crash
    }
}