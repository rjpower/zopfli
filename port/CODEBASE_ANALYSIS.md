This document outlines the structures and functions of the Zopfli C library, in dependency order, with notes on porting them to Rust.

**Assumed `ifdef` Conditions:**

Based on `src/zopfli/util.h` and common compiler features, the following preprocessor defines are assumed to be active:
*   `ZOPFLI_LONGEST_MATCH_CACHE`
*   `ZOPFLI_HASH_SAME`
*   `ZOPFLI_HASH_SAME_HASH`
*   `ZOPFLI_SHORTCUT_LONG_REPETITIONS`
*   `ZOPFLI_LAZY_MATCHING`
*   `ZOPFLI_HAS_BUILTIN_CLZ` (from `src/zopfli/symbols.h`, assuming a GCC compiler version that supports `__builtin_clz` as per the conditional compilation logic in that file and the provided GCC compile command).

All other `ifdef`s are assumed to be set as defined in their respective source files or disabled if not explicitly defined.

**Core Constants (from `src/zopfli/util.h`)**

These constants define fundamental parameters for the Zopfli algorithm. In Rust, they would typically be `const` values.

*   `ZOPFLI_MAX_MATCH`: 258. Maximum LZ77 match length.
*   `ZOPFLI_MIN_MATCH`: 3. Minimum LZ77 match length.
*   `ZOPFLI_NUM_LL`: 288. Number of Literal/Length codes in DEFLATE.
*   `ZOPFLI_NUM_D`: 32. Number of Distance codes in DEFLATE.
*   `ZOPFLI_WINDOW_SIZE`: 32768. LZ77 sliding window size.
*   `ZOPFLI_WINDOW_MASK`: (`ZOPFLI_WINDOW_SIZE` - 1). Mask for window indexing.
*   `ZOPFLI_MASTER_BLOCK_SIZE`: 1000000. Size of master blocks for processing large files. If 0, the entire input is one master block. (Value is 1000000, used in `ZopfliDeflate`).
*   `ZOPFLI_LARGE_FLOAT`: 1e30. A large float value used for initial costs.
*   `ZOPFLI_CACHE_LENGTH`: 8. Length of sublengths to cache for LMC. (Used by `ZopfliLongestMatchCache`).
*   `ZOPFLI_MAX_CHAIN_HITS`: 8192. Maximum number of hash chain hits to check for LZ77.

**Macro `ZOPFLI_APPEND_DATA` (from `src/zopfli/util.h`)**

This macro is used throughout the C code to append an element to a dynamically growing array.

*   **C Signature (conceptual):** `ZOPFLI_APPEND_DATA(value, &array_ptr, &current_size)`
*   **`value`**: The value to append.
*   **`data`**: A pointer to the pointer of the array (e.g., `unsigned short**`).
*   **`size`**: A pointer to the current number of elements in the array (e.g., `size_t*`).
*   **Behavior**:
    *   Checks if `*size` is a power of two (or zero). If so, it reallocates `*data`.
    *   If `*size` was 0, it `malloc`s space for 1 element.
    *   Otherwise, it `realloc`s `*data` to `(*size) * 2` elements.
    *   Assigns `value` to `(*data)[(*size)]`.
    *   Increments `*size`.
*   **Rust Equivalent**: This behavior is largely encapsulated by `Vec<T>`. `Vec::push(value)` will handle appending and reallocation.
*   **Porting Issues**: Direct translation is not needed. Replace usages with `Vec::push()`. Ensure the element types match. The C code may exit on allocation failure; Rust's `Vec` will panic.

## File: src/zopfli/symbols.h

This file contains static inline functions for DEFLATE symbol calculations. In Rust, these would likely be private helper functions within the relevant modules, or public if they form a distinct API. The `static const` tables would become `static` arrays in Rust.

Assuming `ZOPFLI_HAS_BUILTIN_CLZ` is defined, the versions using `__builtin_clz` are active.

### Function: `ZopfliGetDistExtraBits`
*   **C Signature**: `static int ZopfliGetDistExtraBits(int dist)`
*   **`dist`**: The LZ77 distance.
*   **Return Value**: `int`. The number of extra bits required for the given distance in DEFLATE.
*   **Observed Usage**: Calculates extra bits for distances.
*   **Argument Nullability**: `dist` is a scalar, not nullable.
*   **Porting Issues**: Straightforward translation. `__builtin_clz` can be mapped to `leading_zeros()` method on integers in Rust.

### Function: `ZopfliGetDistExtraBitsValue`
*   **C Signature**: `static int ZopfliGetDistExtraBitsValue(int dist)`
*   **`dist`**: The LZ77 distance.
*   **Return Value**: `int`. The value of the extra bits for the given distance.
*   **Observed Usage**: Calculates the value of extra bits for distances.
*   **Argument Nullability**: `dist` is a scalar, not nullable.
*   **Porting Issues**: Straightforward. `__builtin_clz` maps to `leading_zeros()`.

### Function: `ZopfliGetDistSymbol`
*   **C Signature**: `static int ZopfliGetDistSymbol(int dist)`
*   **`dist`**: The LZ77 distance.
*   **Return Value**: `int`. The DEFLATE distance symbol (0-29).
*   **Observed Usage**: Converts an LZ77 distance to its DEFLATE symbol.
*   **Argument Nullability**: `dist` is a scalar, not nullable.
*   **Porting Issues**: Straightforward. `__builtin_clz` maps to `leading_zeros()`.

### Function: `ZopfliGetLengthExtraBits`
*   **C Signature**: `static int ZopfliGetLengthExtraBits(int l)`
*   **`l`**: The LZ77 match length.
*   **Return Value**: `int`. The number of extra bits for the given length.
*   **Observed Usage**: Uses a lookup table `table[259]` to find the number of extra bits.
*   **Argument Nullability**: `l` is a scalar, not nullable.
*   **Porting Issues**: The static table `table` needs to be defined in Rust. Array indexing should be safe (ensure `l` is within bounds if not guaranteed by DEFLATE spec, though `l` seems to be 3-258).

### Function: `ZopfliGetLengthExtraBitsValue`
*   **C Signature**: `static int ZopfliGetLengthExtraBitsValue(int l)`
*   **`l`**: The LZ77 match length.
*   **Return Value**: `int`. The value of the extra bits for the given length.
*   **Observed Usage**: Uses a lookup table `table[259]` to find the value of extra bits.
*   **Argument Nullability**: `l` is a scalar, not nullable.
*   **Porting Issues**: Similar to `ZopfliGetLengthExtraBits`, manage the static table.

### Function: `ZopfliGetLengthSymbol`
*   **C Signature**: `static int ZopfliGetLengthSymbol(int l)`
*   **`l`**: The LZ77 match length.
*   **Return Value**: `int`. The DEFLATE length symbol (257-285).
*   **Observed Usage**: Uses a lookup table `table[259]` to convert an LZ77 length to its DEFLATE symbol.
*   **Argument Nullability**: `l` is a scalar, not nullable.
*   **Porting Issues**: Manage the static table.

### Function: `ZopfliGetLengthSymbolExtraBits`
*   **C Signature**: `static int ZopfliGetLengthSymbolExtraBits(int s)`
*   **`s`**: The DEFLATE length symbol (257-285).
*   **Return Value**: `int`. The number of extra bits associated with this length symbol.
*   **Observed Usage**: Uses a lookup table `table[29]` (indexed by `s - 257`).
*   **Argument Nullability**: `s` is a scalar, not nullable.
*   **Porting Issues**: Manage the static table. Ensure correct indexing.

### Function: `ZopfliGetDistSymbolExtraBits`
*   **C Signature**: `static int ZopfliGetDistSymbolExtraBits(int s)`
*   **`s`**: The DEFLATE distance symbol (0-29).
*   **Return Value**: `int`. The number of extra bits associated with this distance symbol.
*   **Observed Usage**: Uses a lookup table `table[30]`.
*   **Argument Nullability**: `s` is a scalar, not nullable.
*   **Porting Issues**: Manage the static table.

## File: src/zopfli/zopfli.h

### Enum: `ZopfliFormat`
*   **C Definition**:
    ```c
    typedef enum {
      ZOPFLI_FORMAT_GZIP,
      ZOPFLI_FORMAT_ZLIB,
      ZOPFLI_FORMAT_DEFLATE
    } ZopfliFormat;
    ```
*   **Purpose**: Specifies the output compression format.
*   **Rust Equivalent**:
    ```rust
    pub enum ZopfliFormat {
        Gzip,
        Zlib,
        Deflate,
    }
    ```
*   **Porting Issues**: None.

### Structure: `ZopfliOptions`
*   **C Definition**:
    ```c
    typedef struct ZopfliOptions {
      int verbose;
      int verbose_more;
      int numiterations;
      int blocksplitting;
      int blocksplittinglast; // Observed to be unused in zopfli_bin.c, potentially vestigial or for other tools
      int blocksplittingmax;
    } ZopfliOptions;
    ```
*   **Members**:
    *   `verbose`: `int`. If non-zero, print general verbose output.
    *   `verbose_more`: `int`. If non-zero, print more detailed verbose output (e.g., iteration costs).
    *   `numiterations`: `int`. Number of iterations for optimization. Higher is slower but potentially better compression.
    *   `blocksplitting`: `int`. If non-zero, enables block splitting.
    *   `blocksplittinglast`: `int`. If non-zero, enables last block splitting (seems unused in `zopfli_bin.c`).
    *   `blocksplittingmax`: `int`. Maximum number of blocks to split into.
*   **Purpose**: Holds configuration options for Zopfli compression.
*   **Rust Equivalent**:
    ```rust
    pub struct ZopfliOptions {
        pub verbose: bool, // Or i32 if specific int values matter beyond true/false
        pub verbose_more: bool,
        pub num_iterations: i32,
        pub block_splitting: bool,
        // pub block_splitting_last: bool, // Consider omitting if truly unused
        pub block_splitting_max: i32,
    }
    ```
*   **Porting Issues**: `blocksplittinglast` seems unused in the provided `zopfli_bin.c` logic, might be legacy. Confirm its necessity. Integers used as booleans can be mapped to `bool` in Rust.

### Function: `ZopfliInitOptions`
*   **C Signature**: `void ZopfliInitOptions(ZopfliOptions* options)`
*   **`options`**: Non-nullable pointer to a `ZopfliOptions` struct to be initialized.
*   **Purpose**: Initializes a `ZopfliOptions` struct with default values.
    *   `verbose = 0`
    *   `verbose_more = 0`
    *   `numiterations = 15`
    *   `blocksplitting = 1`
    *   `blocksplittinglast = 0`
    *   `blocksplittingmax = 15`
*   **Rust Equivalent**: Typically a `Default::default()` implementation or a `new()` constructor for the `ZopfliOptions` struct.
    ```rust
    impl Default for ZopfliOptions {
        fn default() -> Self {
            ZopfliOptions {
                verbose: false,
                verbose_more: false,
                num_iterations: 15,
                block_splitting: true,
                // block_splitting_last: false,
                block_splitting_max: 15,
            }
        }
    }
    ```
*   **Porting Issues**: None.

## File: src/zopfli/cache.h (and cache.c)
(Active because `ZOPFLI_LONGEST_MATCH_CACHE` is defined)

### Structure: `ZopfliLongestMatchCache`
*   **C Definition**:
    ```c
    typedef struct ZopfliLongestMatchCache {
      unsigned short* length;
      unsigned short* dist;
      unsigned char* sublen;
    } ZopfliLongestMatchCache;
    ```
*   **Members**:
    *   `length`: Pointer to an array of `unsigned short`. Stores the best match length found at each position. Allocated to `blocksize`.
    *   `dist`: Pointer to an array of `unsigned short`. Stores the distance for the best match length at each position. Allocated to `blocksize`.
    *   `sublen`: Pointer to an array of `unsigned char`. Stores (length, dist_low_byte, dist_high_byte) tuples for sub-optimal matches to reconstruct path for LMC. Allocated to `ZOPFLI_CACHE_LENGTH * 3 * blocksize`.
*   **Purpose**: Caches results of `ZopfliFindLongestMatch` to speed up repeated computations over the same block data.
*   **Memory Management**: `length`, `dist`, `sublen` are dynamically allocated in `ZopfliInitCache` and freed in `ZopfliCleanCache`.
*   **Rust Equivalent**:
    ```rust
    pub struct ZopfliLongestMatchCache {
        length: Vec<u16>,
        dist: Vec<u16>,
        sublen: Vec<u8>, // Stores (length-3, dist_lsb, dist_msb) tuples
                        // Access would be sublen[idx * ZOPFLI_CACHE_LENGTH * 3 + ...]
    }
    ```
    The struct would manage its own memory. `blocksize` would be passed to its constructor.
*   **Porting Issues**: Manual memory management needs to be replaced by `Vec<T>`. Access to `sublen` needs careful indexing translation. `ZOPFLI_CACHE_LENGTH` is a constant.

### Function: `ZopfliInitCache`
*   **C Signature**: `void ZopfliInitCache(size_t blocksize, ZopfliLongestMatchCache* lmc)`
*   **`blocksize`**: `size_t`. The size of the data block this cache will be used for.
*   **`lmc`**: Non-nullable pointer to a `ZopfliLongestMatchCache` struct to initialize.
*   **Purpose**: Allocates memory for the cache arrays within `lmc` and initializes them.
    *   `lmc->length` initialized to 1s.
    *   `lmc->dist` initialized to 0s.
    *   `lmc->sublen` initialized to 0s.
*   **Side Effects**: Allocates memory. Exits on failure.
*   **Rust Equivalent**: A constructor `ZopfliLongestMatchCache::new(blocksize: usize) -> Self`.
*   **Porting Issues**: `exit(EXIT_FAILURE)` should be replaced with Rust error handling (e.g., returning `Result` or panicking).

### Function: `ZopfliCleanCache`
*   **C Signature**: `void ZopfliCleanCache(ZopfliLongestMatchCache* lmc)`
*   **`lmc`**: Non-nullable pointer to a `ZopfliLongestMatchCache` struct whose members are to be freed.
*   **Purpose**: Frees the memory allocated for the cache arrays.
*   **Side Effects**: Frees memory.
*   **Rust Equivalent**: Implemented as the `Drop` trait for `ZopfliLongestMatchCache`.
*   **Porting Issues**: None, if Rust's ownership handles it.

### Function: `ZopfliSublenToCache`
*   **C Signature**: `void ZopfliSublenToCache(const unsigned short* sublen, size_t pos, size_t length, ZopfliLongestMatchCache* lmc)`
*   **`sublen`**: Non-nullable pointer to an array of `unsigned short` containing distances for match lengths 3 to `length`. `sublen[k]` is the distance for match length `k`.
*   **`pos`**: `size_t`. The position in the block for which these sublengths are cached.
*   **`length`**: `size_t`. The maximum length for which `sublen` data is valid.
*   **`lmc`**: Non-nullable pointer to the `ZopfliLongestMatchCache`.
*   **Purpose**: Stores a compressed representation of `sublen` (distances for various match lengths) into `lmc->sublen`. It stores up to `ZOPFLI_CACHE_LENGTH` entries, prioritizing changes in distance.
*   **Argument Nullability**: `sublen` and `lmc` are non-nullable.
*   **Porting Issues**: `ZOPFLI_CACHE_LENGTH` (compile-time constant, 0 means disabled). Complex indexing into `lmc->sublen`. Logic for selecting which (length, dist) pairs to store.

### Function: `ZopfliCacheToSublen`
*   **C Signature**: `void ZopfliCacheToSublen(const ZopfliLongestMatchCache* lmc, size_t pos, size_t length, unsigned short* sublen)`
*   **`lmc`**: Non-nullable pointer to the `ZopfliLongestMatchCache`.
*   **`pos`**: `size_t`. The position in the block.
*   **`length`**: `size_t`. The maximum length to reconstruct into `sublen`. (Actually, this `length` parameter seems unused in the `ZopfliMaxCachedSublen` call and might be more of a hint or upper bound for `sublen` array). The function uses `ZopfliMaxCachedSublen` to determine how much to read.
*   **`sublen`**: Non-nullable pointer to an array of `unsigned short` to be filled with distance values.
*   **Purpose**: Reconstructs the `sublen` array (distances for various match lengths) from the cached data in `lmc->sublen`.
*   **Argument Nullability**: `lmc` and `sublen` are non-nullable.
*   **Porting Issues**: Counterpart to `ZopfliSublenToCache`. Complex indexing.

### Function: `ZopfliMaxCachedSublen`
*   **C Signature**: `unsigned ZopfliMaxCachedSublen(const ZopfliLongestMatchCache* lmc, size_t pos, size_t length)`
*   **`lmc`**: Non-nullable pointer to the `ZopfliLongestMatchCache`.
*   **`pos`**: `size_t`. The position in the block.
*   **`length`**: `size_t`. This parameter is marked `(void)length`, indicating it's unused.
*   **Return Value**: `unsigned`. The maximum match length for which distance information is stored in the cache at `pos`. Returns 0 if no valid cache entry.
*   **Observed Usage**: Determines the extent of cached sub-length information.
*   **Argument Nullability**: `lmc` is non-nullable.
*   **Porting Issues**: Unused `length` parameter.

## File: src/zopfli/hash.h (and hash.c)

### Structure: `ZopfliHash`
*   **C Definition (effective, with `ifdefs` resolved)**:
    ```c
    typedef struct ZopfliHash {
      int* head; // Size 65536
      unsigned short* prev; // Size window_size
      int* hashval; // Size window_size
      int val; // Current hash value
      // #ifdef ZOPFLI_HASH_SAME_HASH
      int* head2; // Size 65536
      unsigned short* prev2; // Size window_size
      int* hashval2; // Size window_size
      int val2; // Current hash value for same_hash
      // #endif
      // #ifdef ZOPFLI_HASH_SAME
      unsigned short* same; // Size window_size, stores length of run of identical bytes
      // #endif
    } ZopfliHash;
    ```
*   **Members**: All pointers are dynamically allocated.
    *   `head`: Array mapping hash values to the most recent position (index in `prev`) with that hash.
    *   `prev`: Array forming linked lists for each hash bucket. `prev[pos & ZOPFLI_WINDOW_MASK]` stores the previous position with the same hash.
    *   `hashval`: Stores the hash value computed at each window position. Used to resolve hash collisions.
    *   `val`: The current rolling hash value.
    *   `head2`, `prev2`, `hashval2`, `val2`: Similar to above, but for a secondary hash used when `ZOPFLI_HASH_SAME_HASH` is enabled. This secondary hash incorporates `same[]` data.
    *   `same`: Stores the length of the sequence of identical bytes starting at each position, minus `ZOPFLI_MIN_MATCH`.
*   **Purpose**: Implements a rolling hash to find LZ77 matches quickly.
*   **Rust Equivalent**:
    ```rust
    pub struct ZopfliHash {
        head: Vec<i32>,
        prev: Vec<u16>,
        hash_val: Vec<i32>, // Renamed from hashval to avoid keyword conflict
        val: i32,
        // if ZOPFLI_HASH_SAME_HASH
        head2: Vec<i32>,
        prev2: Vec<u16>,
        hash_val2: Vec<i32>,
        val2: i32,
        // endif
        // if ZOPFLI_HASH_SAME
        same: Vec<u16>,
        // endif
        // window_size: usize, // Might be stored if needed by methods
    }
    ```
    The struct would manage its own memory. Constants like 65536 would be used for `Vec` initialization.
*   **Porting Issues**: Manual memory management to `Vec`. The fixed size 65536 for `head` arrays. `window_size` parameter for other arrays.

### Function: `ZopfliAllocHash`
*   **C Signature**: `void ZopfliAllocHash(size_t window_size, ZopfliHash* h)`
*   **`window_size`**: `size_t`. The size of the sliding window (e.g., `ZOPFLI_WINDOW_SIZE`).
*   **`h`**: Non-nullable pointer to a `ZopfliHash` struct.
*   **Purpose**: Allocates memory for all internal arrays of the `ZopfliHash` struct.
*   **Side Effects**: Allocates memory. Does not appear to `exit()` on failure here, but callers might.
*   **Rust Equivalent**: Constructor `ZopfliHash::new(window_size: usize) -> Self`.
*   **Porting Issues**: Error handling for allocation.

### Function: `ZopfliResetHash`
*   **C Signature**: `void ZopfliResetHash(size_t window_size, ZopfliHash* h)`
*   **`window_size`**: `size_t`. The size of the sliding window.
*   **`h`**: Non-nullable pointer to a `ZopfliHash` struct.
*   **Purpose**: Initializes/resets the hash table fields to default values (e.g., -1 for heads, identity for prev links).
*   **Rust Equivalent**: A method `reset(&mut self, window_size: usize)` on `ZopfliHash`. Could be part of `new`.
*   **Porting Issues**: None.

### Function: `ZopfliCleanHash`
*   **C Signature**: `void ZopfliCleanHash(ZopfliHash* h)`
*   **`h`**: Non-nullable pointer to a `ZopfliHash` struct.
*   **Purpose**: Frees all dynamically allocated memory within the `ZopfliHash` struct.
*   **Side Effects**: Frees memory.
*   **Rust Equivalent**: Implemented by the `Drop` trait for `ZopfliHash`.
*   **Porting Issues**: None if Rust ownership handles it.

### Function: `ZopfliUpdateHash` (static in .c, but declared in .h implies broader use)
*   **C Signature**: `void ZopfliUpdateHash(const unsigned char* array, size_t pos, size_t end, ZopfliHash* h)`
*   **`array`**: Non-nullable pointer to the input byte array.
*   **`pos`**: `size_t`. Current position in `array` to update the hash for.
*   **`end`**: `size_t`. End position (exclusive) of the input data in `array`. Used for boundary checks.
*   **`h`**: Non-nullable pointer to the `ZopfliHash` struct to be updated.
*   **Purpose**: Updates the rolling hash state with the byte at `array[pos + ZOPFLI_MIN_MATCH - 1]` (or 0 if out of bounds) and inserts the current `pos` into the hash table. Also updates `same` array and secondary hash if enabled.
*   **Side Effects**: Modifies `h`.
*   **Constants**: `HASH_SHIFT`, `HASH_MASK` (local to `hash.c`). `ZOPFLI_WINDOW_MASK`, `ZOPFLI_MIN_MATCH`.
*   **Rust Equivalent**: Method `update(&mut self, array: &[u8], pos: usize, end: usize)`.
*   **Porting Issues**: Careful translation of hash update logic and indexing. `pos + ZOPFLI_MIN_MATCH <= end ? array[pos + ZOPFLI_MIN_MATCH - 1] : 0` boundary condition.

### Function: `ZopfliWarmupHash`
*   **C Signature**: `void ZopfliWarmupHash(const unsigned char* array, size_t pos, size_t end, ZopfliHash* h)`
*   **`array`**: Non-nullable pointer to input byte array.
*   **`pos`**: `size_t`. Starting position in `array` for warmup.
*   **`end`**: `size_t`. End position (exclusive) in `array`.
*   **`h`**: Non-nullable pointer to `ZopfliHash` struct.
*   **Purpose**: Initializes the rolling hash `h->val` with the first few bytes (`ZOPFLI_MIN_MATCH - 1`, effectively 2 bytes) of the data before starting main processing.
*   **Side Effects**: Modifies `h->val`.
*   **Rust Equivalent**: Method `warmup(&mut self, array: &[u8], pos: usize, end: usize)`.
*   **Porting Issues**: Boundary checks `pos + 1 < end`.

## File: src/zopfli/katajainen.h (and katajainen.c)

### Function: `ZopfliLengthLimitedCodeLengths`
*   **C Signature**: `int ZopfliLengthLimitedCodeLengths(const size_t* frequencies, int n, int maxbits, unsigned* bitlengths)`
*   **`frequencies`**: Non-nullable pointer to an array of symbol frequencies. Size `n`.
*   **`n`**: `int`. Number of symbols.
*   **`maxbits`**: `int`. Maximum allowed bit length for any code.
*   **`bitlengths`**: Non-nullable pointer to an output array of `unsigned int`, size `n`. Will be filled with calculated bit lengths.
*   **Return Value**: `int`. 0 on success, 1 on error (e.g., not possible to satisfy `maxbits` or too many symbols).
*   **Purpose**: Implements the Katajainen algorithm for constructing length-limited Huffman codes. Given symbol frequencies, it calculates the bit lengths for each symbol such that no length exceeds `maxbits`.
*   **Side Effects**: Modifies `bitlengths`. Allocates temporary memory internally (`leaves`, `nodes`, `lists`).
*   **Internal types**: `Node`, `NodePool` (local to .c file).
*   **Rust Equivalent**: A public function. The internal `Node` and `NodePool` would likely be helper structs, possibly in a private module or embedded in the function's scope if closures can manage it.
*   **Porting Issues**:
    *   Manual memory management for `leaves`, `nodes`, `lists`.
    *   `qsort` used on `leaves`. Rust would use `slice::sort_by`.
    *   Pointer-heavy logic with `Node` structures and linked lists (`tail`). This needs careful translation to Rust's ownership model, possibly using `Vec` indices or `Box<Node>` with `Option<Box<Node>>` for tails if direct graph structure is kept, or an arena allocator.
    *   The algorithm itself is complex.
    *   Error conditions (return 1) should be mapped to `Result<_, Error>`.
    *   Integer type conversions (`size_t` to `int` for `numsymbols`, `maxbits`).

## File: src/zopfli/tree.h (and tree.c)

### Function: `ZopfliLengthsToSymbols`
*   **C Signature**: `void ZopfliLengthsToSymbols(const unsigned* lengths, size_t n, unsigned maxbits, unsigned* symbols)`
*   **`lengths`**: Non-nullable pointer to an array of code bit lengths. Size `n`.
*   **`n`**: `size_t`. Number of symbols.
*   **`maxbits`**: `unsigned`. Maximum bit length present in `lengths`.
*   **`symbols`**: Non-nullable pointer to an output array of `unsigned int`. Size `n`. Will be filled with the canonical Huffman codes.
*   **Purpose**: Converts a list of Huffman code lengths into the actual Huffman codes (symbols) according to the canonical Huffman code generation rules.
*   **Side Effects**: Modifies `symbols`. Allocates temporary memory (`bl_count`, `next_code`).
*   **Rust Equivalent**: Public function.
*   **Porting Issues**: Manual memory management for `bl_count` and `next_code`.

### Function: `ZopfliCalculateEntropy`
*   **C Signature**: `void ZopfliCalculateEntropy(const size_t* count, size_t n, double* bitlengths)`
*   **`count`**: Non-nullable pointer to an array of symbol counts/frequencies. Size `n`.
*   **`n`**: `size_t`. Number of symbols.
*   **`bitlengths`**: Non-nullable pointer to an output array of `double`. Size `n`. Will be filled with calculated ideal bit lengths (entropy).
*   **Purpose**: Calculates the Shannon entropy for each symbol, which represents the theoretical minimum average bit length. `bitlengths[i] = log2(sum_counts) - log2(count[i])`.
*   **Side Effects**: Modifies `bitlengths`. Uses `log` from `math.h`.
*   **Constants**: `kInvLog2 = 1.0 / log(2.0)`.
*   **Rust Equivalent**: Public function. `f64::log2` or `f64::ln` can be used.
*   **Porting Issues**: Floating point arithmetic. `count[i] == 0` case handled by assigning `log2sum`.

### Function: `ZopfliCalculateBitLengths`
*   **C Signature**: `void ZopfliCalculateBitLengths(const size_t* count, size_t n, int maxbits, unsigned* bitlengths)`
*   **`count`**: Non-nullable pointer to an array of symbol counts/frequencies. Size `n`.
*   **`n`**: `size_t`. Number of symbols.
*   **`maxbits`**: `int`. Maximum allowed bit length for any code.
*   **`bitlengths`**: Non-nullable pointer to an output array of `unsigned int`. Size `n`. Will be filled with calculated bit lengths.
*   **Purpose**: Calculates Huffman code lengths for symbols given their frequencies, respecting `maxbits`. It calls `ZopfliLengthLimitedCodeLengths`.
*   **Side Effects**: Modifies `bitlengths`. The `assert(!error)` suggests that failure from `ZopfliLengthLimitedCodeLengths` is considered a critical error.
*   **Rust Equivalent**: Public function.
*   **Porting Issues**: Error handling from `ZopfliLengthLimitedCodeLengths` should be propagated (e.g. using `Result`).

## File: src/zopfli/lz77.h (and lz77.c)

### Structure: `ZopfliLZ77Store`
*   **C Definition**:
    ```c
    typedef struct ZopfliLZ77Store {
      unsigned short* litlens; // Literal value or match length
      unsigned short* dists;   // Match distance (0 for literals)
      size_t size;             // Current number of LZ77 items stored
      const unsigned char* data; // Pointer to original input data (for reference, not owned)
      size_t* pos;             // Original position in 'data' for each LZ77 item
      // Cache of DEFLATE symbols and counts, updated by ZopfliStoreLitLenDist
      unsigned short* ll_symbol;
      unsigned short* d_symbol;
      size_t* ll_counts; // Histograms, possibly in blocks
      size_t* d_counts;  // Histograms, possibly in blocks
    } ZopfliLZ77Store;
    ```
*   **Members**:
    *   `litlens`, `dists`, `pos`, `ll_symbol`, `d_symbol`, `ll_counts`, `d_counts`: Dynamically allocated arrays, grown using `ZOPFLI_APPEND_DATA`.
    *   `size`: Number of valid entries in `litlens`, `dists`, `pos`, etc.
    *   `data`: A non-owning pointer to the original input byte array.
*   **Purpose**: Stores a sequence of LZ77 encoded literal/length-distance pairs. Also maintains pre-calculated symbol representations and histograms for efficiency.
*   **Memory Management**: Internal arrays are managed by `ZopfliInitLZ77Store`, `ZopfliCleanLZ77Store`, `ZopfliStoreLitLenDist`, etc.
*   **Rust Equivalent**:
    ```rust
    pub struct ZopfliLZ77Store<'a> {
        litlens: Vec<u16>,
        dists: Vec<u16>,
        // size is implicit Vec::len()
        data: &'a [u8], // Non-owning slice
        pos: Vec<usize>,
        ll_symbol: Vec<u16>,
        d_symbol: Vec<u16>,
        ll_counts: Vec<usize>,
        d_counts: Vec<usize>,
    }
    ```
    The `size` member becomes implicit from `Vec::len()`. `'a` lifetime for `data`.
*   **Porting Issues**: `ZOPFLI_APPEND_DATA` logic for growing arrays. The `ll_counts` and `d_counts` are interesting: they seem to be cumulative or block-based histograms. Their update logic in `ZopfliStoreLitLenDist` and usage in `ZopfliLZ77GetHistogramAt` needs careful porting. The allocation strategy (doubling) of `ZOPFLI_APPEND_DATA` for these count arrays implies they grow with `store->size` but are indexed in blocks of `ZOPFLI_NUM_LL` or `ZOPFLI_NUM_D`.

### Function: `ZopfliInitLZ77Store`
*   **C Signature**: `void ZopfliInitLZ77Store(const unsigned char* data, ZopfliLZ77Store* store)`
*   **`data`**: Non-nullable pointer to the original input data. `store` will not own this.
*   **`store`**: Non-nullable pointer to a `ZopfliLZ77Store` to initialize.
*   **Purpose**: Initializes a `ZopfliLZ77Store` struct. Sets pointers to 0 (NULL) and `size` to 0. Assigns `store->data`.
*   **Rust Equivalent**: `ZopfliLZ77Store::new(data: &'a [u8]) -> Self`.
*   **Porting Issues**: None.

### Function: `ZopfliCleanLZ77Store`
*   **C Signature**: `void ZopfliCleanLZ77Store(ZopfliLZ77Store* store)`
*   **`store`**: Non-nullable pointer to a `ZopfliLZ77Store`.
*   **Purpose**: Frees all dynamically allocated arrays within `store`.
*   **Rust Equivalent**: Implemented by the `Drop` trait for `ZopfliLZ77Store`.
*   **Porting Issues**: None if Rust ownership handles it.

### Function: `ZopfliCopyLZ77Store`
*   **C Signature**: `void ZopfliCopyLZ77Store(const ZopfliLZ77Store* source, ZopfliLZ77Store* dest)`
*   **`source`**: Non-nullable pointer to the source `ZopfliLZ77Store`.
*   **`dest`**: Non-nullable pointer to the destination `ZopfliLZ77Store`.
*   **Purpose**: Performs a deep copy of `source` into `dest`. `dest` is cleaned first.
*   **Side Effects**: Allocates memory for `dest`. Exits on failure.
*   **Rust Equivalent**: Implement the `Clone` trait for `ZopfliLZ77Store`.
*   **Porting Issues**: `exit(-1)` on allocation failure. `CeilDiv` is a helper. The allocation sizes for `ll_counts` and `d_counts` are rounded up to multiples of `ZOPFLI_NUM_LL` and `ZOPFLI_NUM_D`.

### Function: `ZopfliStoreLitLenDist`
*   **C Signature**: `void ZopfliStoreLitLenDist(unsigned short length, unsigned short dist, size_t pos, ZopfliLZ77Store* store)`
*   **`length`**: `unsigned short`. Literal value (if `dist` is 0) or match length.
*   **`dist`**: `unsigned short`. Match distance (0 for literals).
*   **`pos`**: `size_t`. Original position in input data for this item.
*   **`store`**: Non-nullable pointer to `ZopfliLZ77Store` to append to.
*   **Purpose**: Appends a new literal or length-distance pair to the `store`. Updates `litlens`, `dists`, `pos`, `ll_symbol`, `d_symbol`, and the histograms `ll_counts`, `d_counts`.
*   **Side Effects**: Modifies `store` using `ZOPFLI_APPEND_DATA`.
*   **Porting Issues**: Complex logic for updating `ll_counts` and `d_counts`. These counts are appended in blocks if `origsize` (before append) is a multiple of `ZOPFLI_NUM_LL` (or `_D`), copying previous block's final counts. This implies a prefix-sum like behavior for histograms across blocks.

### Function: `ZopfliAppendLZ77Store`
*   **C Signature**: `void ZopfliAppendLZ77Store(const ZopfliLZ77Store* store, ZopfliLZ77Store* target)`
*   **`store`**: Non-nullable pointer to the source `ZopfliLZ77Store`.
*   **`target`**: Non-nullable pointer to the target `ZopfliLZ77Store`.
*   **Purpose**: Appends all items from `store` to `target` using `ZopfliStoreLitLenDist`.
*   **Side Effects**: Modifies `target`.
*   **Rust Equivalent**: A method like `append(&mut self, other: &ZopfliLZ77Store)`.
*   **Porting Issues**: Relies on `ZopfliStoreLitLenDist` being correct.

### Function: `ZopfliLZ77GetByteRange`
*   **C Signature**: `size_t ZopfliLZ77GetByteRange(const ZopfliLZ77Store* lz77, size_t lstart, size_t lend)`
*   **`lz77`**: Non-nullable pointer to `ZopfliLZ77Store`.
*   **`lstart`**: `size_t`. Start index (inclusive) in the LZ77 store.
*   **`lend`**: `size_t`. End index (exclusive) in the LZ77 store.
*   **Return Value**: `size_t`. The total number of uncompressed bytes represented by the LZ77 items from `lstart` to `lend-1`.
*   **Observed Usage**: Calculates original data length for a segment of LZ77 items.
*   **Porting Issues**: Correctly handles empty ranges. Accesses `lz77->pos`, `lz77->dists`, `lz77->litlens`.

### Function: `ZopfliLZ77GetHistogram`
*   **C Signature**: `void ZopfliLZ77GetHistogram(const ZopfliLZ77Store* lz77, size_t lstart, size_t lend, size_t* ll_counts, size_t* d_counts)`
*   **`lz77`**: Non-nullable pointer to `ZopfliLZ77Store`.
*   **`lstart`**: `size_t`. Start index in LZ77 store.
*   **`lend`**: `size_t`. End index in LZ77 store.
*   **`ll_counts`**: Non-nullable output array (size `ZOPFLI_NUM_LL`) for literal/length symbol counts.
*   **`d_counts`**: Non-nullable output array (size `ZOPFLI_NUM_D`) for distance symbol counts.
*   **Purpose**: Calculates histograms of literal/length and distance symbols for the LZ77 items in the range `[lstart, lend)`.
*   **Side Effects**: Fills `ll_counts` and `d_counts`.
*   **Optimization**: If the range is large, it uses precomputed cumulative histograms (`ZopfliLZ77GetHistogramAt`) from `lz77->ll_counts` and `lz77->d_counts`. Otherwise, it iterates.
*   **Porting Issues**: The histogram optimization relies on the specific structure of `lz77->ll_counts` and `lz77->d_counts` being cumulative or block-based prefix sums. `ZopfliLZ77GetHistogramAt` (static helper) needs careful porting.

### Structure: `ZopfliBlockState`
*   **C Definition**:
    ```c
    typedef struct ZopfliBlockState {
      const ZopfliOptions* options; // Non-owning
      // #ifdef ZOPFLI_LONGEST_MATCH_CACHE
      ZopfliLongestMatchCache* lmc; // Owned if add_lmc was true in init
      // #endif
      size_t blockstart; // Start of current data block in original input
      size_t blockend;   // End of current data block in original input
    } ZopfliBlockState;
    ```
*   **Members**:
    *   `options`: Non-owning pointer to `ZopfliOptions`.
    *   `lmc`: Pointer to `ZopfliLongestMatchCache`. May be NULL if `add_lmc` was false. Owned by this struct if allocated.
    *   `blockstart`, `blockend`: Define the current processing window within the larger input.
*   **Purpose**: Holds state related to processing a single block of data, including options and potentially a longest match cache.
*   **Rust Equivalent**:
    ```rust
    pub struct ZopfliBlockState<'a> {
        options: &'a ZopfliOptions, // Non-owning reference
        lmc: Option<ZopfliLongestMatchCache>, // Owned
        block_start: usize,
        block_end: usize,
    }
    ```
*   **Porting Issues**: Ownership of `lmc`.

### Function: `ZopfliInitBlockState`
*   **C Signature**: `void ZopfliInitBlockState(const ZopfliOptions* options, size_t blockstart, size_t blockend, int add_lmc, ZopfliBlockState* s)`
*   **`options`**: Non-nullable pointer to `ZopfliOptions`.
*   **`blockstart`**: `size_t`.
*   **`blockend`**: `size_t`.
*   **`add_lmc`**: `int`. If true (non-zero), allocate and initialize `s->lmc`.
*   **`s`**: Non-nullable pointer to `ZopfliBlockState` to initialize.
*   **Purpose**: Initializes `ZopfliBlockState`. If `add_lmc` is true and `ZOPFLI_LONGEST_MATCH_CACHE` is defined, it allocates and initializes `s->lmc`.
*   **Side Effects**: Potentially allocates `s->lmc`.
*   **Rust Equivalent**: `ZopfliBlockState::new(options: &'a ZopfliOptions, block_start: usize, block_end: usize, add_lmc: bool) -> Self`.
*   **Porting Issues**: Conditional allocation of LMC.

### Function: `ZopfliCleanBlockState`
*   **C Signature**: `void ZopfliCleanBlockState(ZopfliBlockState* s)`
*   **`s`**: Non-nullable pointer to `ZopfliBlockState`.
*   **Purpose**: Cleans up resources in `ZopfliBlockState`, primarily freeing `s->lmc` if it was allocated.
*   **Side Effects**: Frees `s->lmc`.
*   **Rust Equivalent**: Implemented by `Drop` trait for `ZopfliBlockState`.
*   **Porting Issues**: None if Rust ownership used.

### Function: `ZopfliFindLongestMatch`
*   **C Signature**: `void ZopfliFindLongestMatch(ZopfliBlockState *s, const ZopfliHash* h, const unsigned char* array, size_t pos, size_t size, size_t limit, unsigned short* sublen, unsigned short* distance, unsigned short* length)`
*   **`s`**: Non-nullable pointer to `ZopfliBlockState`. Used for LMC and block boundaries.
*   **`h`**: Non-nullable pointer to `ZopfliHash` (the hash table).
*   **`array`**: Non-nullable pointer to input byte array.
*   **`pos`**: `size_t`. Current position in `array` to find match for.
*   **`size`**: `size_t`. Total size of `array`.
*   **`limit`**: `size_t`. Maximum match length to consider (e.g., `ZOPFLI_MAX_MATCH`).
*   **`sublen`**: Nullable pointer to an array of `unsigned short` (size `limit`+1 or 259). If non-NULL, `sublen[k]` will be filled with the distance of the best match of length `k`.
*   **`distance`**: Non-nullable pointer to `unsigned short`. Output: distance of the overall best match.
*   **`length`**: Non-nullable pointer to `unsigned short`. Output: length of the overall best match.
*   **Purpose**: Finds the longest match for data starting at `pos` in `array`. Uses the hash table `h` and potentially the LMC from `s`. If `sublen` is provided, it records distances for all match lengths up to the best one.
*   **Side Effects**: Modifies `*distance`, `*length`, and `sublen` if provided. Updates LMC if conditions met.
*   **Rust Equivalent**: A method or function. `sublen` could be `Option<&mut [u16]>`. `distance` and `length` would be part of a returned struct or tuple.
*   **Porting Issues**:
    *   LMC logic (`TryGetFromLongestMatchCache`, `StoreInLongestMatchCache`) is conditional.
    *   Hash chain traversal (`ZOPFLI_MAX_CHAIN_HITS`).
    *   `ZOPFLI_HASH_SAME` and `ZOPFLI_HASH_SAME_HASH` macros affect logic.
    *   Pointer arithmetic in `GetMatch` (optimized byte comparison). Rust's `slice::starts_with` or manual iteration.
    *   The `GetMatch` function has architecture-specific optimizations (sizeof size_t/unsigned int). This can be tricky to port efficiently and safely. Standard library slice comparisons are often optimized.

### Function: `ZopfliVerifyLenDist`
*   **C Signature**: `void ZopfliVerifyLenDist(const unsigned char* data, size_t datasize, size_t pos, unsigned short dist, unsigned short length)`
*   **`data`**: Non-nullable pointer to input byte array.
*   **`datasize`**: `size_t`. Total size of `data`.
*   **`pos`**: `size_t`. Position of the match.
*   **`dist`**: `unsigned short`. Distance of the match.
*   **`length`**: `unsigned short`. Length of the match.
*   **Purpose**: Debugging function. Asserts that the match `(length, dist)` at `pos` is valid (i.e., `data[pos+i] == data[pos-dist+i]`).
*   **Side Effects**: Calls `assert`.
*   **Rust Equivalent**: A debug-only function using `debug_assert!`.
*   **Porting Issues**: None.

### Function: `ZopfliLZ77Greedy`
*   **C Signature**: `void ZopfliLZ77Greedy(ZopfliBlockState* s, const unsigned char* in, size_t instart, size_t inend, ZopfliLZ77Store* store, ZopfliHash* h)`
*   **`s`**: Non-nullable pointer to `ZopfliBlockState`.
*   **`in`**: Non-nullable pointer to input byte array.
*   **`instart`**: `size_t`. Start position in `in`.
*   **`inend`**: `size_t`. End position in `in`.
*   **`store`**: Non-nullable pointer to `ZopfliLZ77Store` to fill with LZ77 codes.
*   **`h`**: Non-nullable pointer to `ZopfliHash` (hash table to use).
*   **Purpose**: Performs greedy LZ77 encoding on the input data range `[instart, inend)` and stores results in `store`.
*   **Side Effects**: Modifies `store` and `h`.
*   **Algorithm**: Iterates through input, calls `ZopfliFindLongestMatch` at each position. If `ZOPFLI_LAZY_MATCHING` is defined, it implements lazy matching (checks if a better match is found at next position).
*   **Rust Equivalent**: A function or method.
*   **Porting Issues**: `ZOPFLI_LAZY_MATCHING` logic. Uses `ZopfliResetHash`, `ZopfliWarmupHash`, `ZopfliUpdateHash`, `ZopfliFindLongestMatch`, `ZopfliStoreLitLenDist`. `GetLengthScore` is a static helper.

## File: src/zopfli/squeeze.h (and squeeze.c)
This file implements the "squeezing" part of Zopfli, which iteratively refines the LZ77 output using dynamic programming and statistical cost models.

### Function: `ZopfliLZ77Optimal`
*   **C Signature**: `void ZopfliLZ77Optimal(ZopfliBlockState *s, const unsigned char* in, size_t instart, size_t inend, int numiterations, ZopfliLZ77Store* store)`
*   **`s`**: Non-nullable pointer to `ZopfliBlockState`.
*   **`in`**: Non-nullable pointer to input byte array.
*   **`instart`**: `size_t`. Start position in `in`.
*   **`inend`**: `size_t`. End position in `in`.
*   **`numiterations`**: `int`. Number of iterations for optimization.
*   **`store`**: Non-nullable pointer to `ZopfliLZ77Store`. Output: the optimized LZ77 sequence.
*   **Purpose**: Performs an iterative optimization of LZ77 coding. In each iteration, it:
    1.  Calculates symbol statistics (frequencies) from the current LZ77 sequence.
    2.  Uses these statistics to define a cost model for LZ77 symbols.
    3.  Finds the shortest path (lowest cost LZ77 sequence) through the data using dynamic programming (`GetBestLengths`).
    4.  The new LZ77 sequence from the shortest path becomes the input for the next iteration.
    The best LZ77 sequence found across all iterations is stored in `store`.
*   **Side Effects**: Modifies `store`. Allocates significant temporary memory (`length_array`, `path`, `costs`, `currentstore`, `hash`, stats structs).
*   **Internal types/logic**:
    *   `SymbolStats`: struct to hold symbol frequencies and calculated bit costs.
    *   `RanState`: for randomizing frequencies if optimization gets stuck.
    *   `CostModelFun` (function pointer type), `GetCostStat` (cost model based on `SymbolStats`).
    *   `GetBestLengths`: dynamic programming core.
    *   `TraceBackwards`: reconstructs LZ77 path from `GetBestLengths` result.
    *   `FollowPath`: converts path into `ZopfliLZ77Store`.
*   **Rust Equivalent**: A complex function. Internal structs and logic would be part of its module.
*   **Porting Issues**:
    *   High complexity. Many helper functions and structs (e.g., `SymbolStats`).
    *   Extensive dynamic memory allocation.
    *   Function pointers (`CostModelFun`) map to closures or trait objects in Rust.
    *   Floating point arithmetic for costs.
    *   `ZOPFLI_SHORTCUT_LONG_REPETITIONS` in `GetBestLengths`.
    *   Randomization logic for escaping local optima.
    *   Verbose output options.

### Function: `ZopfliLZ77OptimalFixed`
*   **C Signature**: `void ZopfliLZ77OptimalFixed(ZopfliBlockState *s, const unsigned char* in, size_t instart, size_t inend, ZopfliLZ77Store* store)`
*   **Arguments**: Same as `ZopfliLZ77Optimal` except `numiterations`.
*   **Purpose**: Similar to `ZopfliLZ77Optimal` but uses a fixed cost model (`GetCostFixed`) based on standard DEFLATE fixed Huffman tree costs, rather than iteratively derived statistical costs. Performs one pass of dynamic programming.
*   **Side Effects**: Modifies `store`. Allocates temporary memory.
*   **Rust Equivalent**: Similar structure to `ZopfliLZ77Optimal` but simpler due to fixed cost model.
*   **Porting Issues**: Shares much of the DP machinery with `ZopfliLZ77Optimal`.

## File: src/zopfli/blocksplitter.h (and blocksplitter.c)

### Function: `ZopfliBlockSplitLZ77`
*   **C Signature**: `void ZopfliBlockSplitLZ77(const ZopfliOptions* options, const ZopfliLZ77Store* lz77, size_t maxblocks, size_t** splitpoints, size_t* npoints)`
*   **`options`**: Non-nullable pointer to `ZopfliOptions`.
*   **`lz77`**: Non-nullable pointer to an existing `ZopfliLZ77Store` (e.g., from a greedy pass).
*   **`maxblocks`**: `size_t`. Maximum number of blocks to split into. If 0, interpreted as no limit by some internal logic, but the loop for splitting breaks if `numblocks >= maxblocks`.
*   **`splitpoints`**: Non-nullable pointer to `size_t*`. Output: an array of indices into the `lz77` store where blocks should be split. Caller must free.
*   **`npoints`**: Non-nullable pointer to `size_t`. Output: number of split points in `*splitpoints`.
*   **Purpose**: Finds optimal DEFLATE block split points for a given LZ77 sequence. It iteratively finds the split that yields the best compression improvement until `maxblocks` is reached or no further improvement is found.
*   **Side Effects**: Allocates `*splitpoints` using `ZOPFLI_APPEND_DATA`. Allocates temporary `done` array.
*   **Internal types/logic**:
    *   `FindMinimumFun` (function pointer type), `FindMinimum` (ternary search / brute force).
    *   `EstimateCost` (calls `ZopfliCalculateBlockSizeAutoType`).
    *   `SplitCostContext`, `SplitCost` (for `FindMinimum`).
    *   `AddSorted`: helper to insert split points.
*   **Rust Equivalent**: A function that returns `Vec<usize>` for split points.
*   **Porting Issues**:
    *   `ZOPFLI_APPEND_DATA` for `splitpoints`.
    *   `FindMinimum` uses a combination of ternary search-like division and brute force for small ranges.
    *   Callback pattern (`FindMinimumFun`) for `FindMinimum`.
    *   Verbose printing logic.
    *   `exit(-1)` in `AddSorted` if `ZOPFLI_APPEND_DATA` fails (not explicit in `AddSorted`, but `malloc` inside `ZOPFLI_APPEND_DATA` can fail). Original `AddSorted` does not have `exit(-1)`. `done` allocation could fail.

### Function: `ZopfliBlockSplit`
*   **C Signature**: `void ZopfliBlockSplit(const ZopfliOptions* options, const unsigned char* in, size_t instart, size_t inend, size_t maxblocks, size_t** splitpoints, size_t* npoints)`
*   **`in`**: Non-nullable pointer to raw input data.
*   **`instart`, `inend`**: Range in `in` to process.
*   **Other args**: Same as `ZopfliBlockSplitLZ77`.
*   **`splitpoints`**: Output array of byte offsets in the original `in` data.
*   **`npoints`**: Number of points in `*splitpoints`.
*   **Purpose**: Top-level block splitting function.
    1.  Performs a greedy LZ77 pass on `in` to get an initial `ZopfliLZ77Store`.
    2.  Calls `ZopfliBlockSplitLZ77` on this store to get LZ77-based split points.
    3.  Converts these LZ77 indices back to byte offsets in the original `in` data.
*   **Side Effects**: Allocates `*splitpoints`. Allocates temporary `lz77splitpoints`, `ZopfliLZ77Store`, `ZopfliHash`.
*   **Rust Equivalent**: Function returning `Vec<usize>`.
*   **Porting Issues**: Manages several temporary Zopfli structures. Conversion from LZ77 indices to byte offsets.

### Function: `ZopfliBlockSplitSimple`
*   **C Signature**: `void ZopfliBlockSplitSimple(const unsigned char* in, size_t instart, size_t inend, size_t blocksize, size_t** splitpoints, size_t* npoints)`
*   **`in`**: Non-nullable pointer to raw input data. (Marked `(void)in`, so content not used, only bounds).
*   **`instart`, `inend`**: Range in `in`.
*   **`blocksize`**: `size_t`. Desired size for each block.
*   **`splitpoints`, `npoints`**: Output as above (byte offsets).
*   **Purpose**: A much simpler block splitting: splits the input range `[instart, inend)` into fixed-size chunks of `blocksize` bytes.
*   **Side Effects**: Allocates `*splitpoints`.
*   **Rust Equivalent**: Function returning `Vec<usize>`.
*   **Porting Issues**: `ZOPFLI_APPEND_DATA`. Straightforward logic.

## File: src/zopfli/deflate.h (and deflate.c)
This file contains the core DEFLATE formatting logic.

### Static Helper Functions (in deflate.c)
*   `AddBit`, `AddBits`, `AddHuffmanBits`: Low-level functions to append bits to the output byte stream (`unsigned char** out, size_t* outsize`). `*bp` is the bit pointer (0-7).
    *   **Porting Issues**: Bit-level manipulation. In Rust, a dedicated bit stream writer struct would be idiomatic. `ZOPFLI_APPEND_DATA` used for `out`.
*   `PatchDistanceCodesForBuggyDecoders`: Modifies distance code lengths to ensure at least two distance codes are present if any are used, for compatibility.
*   `EncodeTree`, `CalculateTreeSize`: Handle encoding of Huffman tree descriptions (HLIT, HDIST, HCLEN) and calculating their bit size. Uses RLE (run-length encoding) for code lengths (symbols 16, 17, 18).
    *   **Porting Issues**: Complex DEFLATE spec logic. Temporary RLE arrays.
*   `AddDynamicTree`: Selects best RLE strategy (use_16/17/18 flags) and calls `EncodeTree`.
*   `AddLZ77Data`: Encodes LZ77 literals and length/distance pairs using provided Huffman codes.
*   `GetFixedTree`: Provides the bit lengths for the DEFLATE fixed Huffman tree.
*   `CalculateBlockSymbolSizeSmall`, `CalculateBlockSymbolSizeGivenCounts`, `CalculateBlockSymbolSize`: Calculate the compressed size of LZ77 data given Huffman code lengths.
*   `OptimizeHuffmanForRle`, `TryOptimizeHuffmanForRle`: Heuristics to adjust Huffman code lengths to be more RLE-compressible for the tree description.
*   `GetDynamicLengths`: Calculates dynamic Huffman tree lengths for a block.
*   `AddNonCompressedBlock`: Creates a DEFLATE non-compressed block (BTYPE 00).
*   `AddLZ77Block`: Creates a DEFLATE compressed block (BTYPE 01 or 10).
*   `AddLZ77BlockAutoType`: Chooses among BTYPE 00, 01, or 10 for a block and adds it. May re-run LZ77 optimal fixed pass if fixed tree is chosen and seems beneficial.

### Function: `ZopfliCalculateBlockSize`
*   **C Signature**: `double ZopfliCalculateBlockSize(const ZopfliLZ77Store* lz77, size_t lstart, size_t lend, int btype)`
*   **`lz77`**: Non-nullable pointer to `ZopfliLZ77Store`.
*   **`lstart`, `lend`**: Range within `lz77` for this block.
*   **`btype`**: `int`. DEFLATE block type (0, 1, or 2).
*   **Return Value**: `double`. Estimated compressed size in bits for this block if encoded with `btype`.
*   **Purpose**: Estimates the compressed size of a DEFLATE block.
*   **Porting Issues**: Calls many static helpers. Floating point return.

### Function: `ZopfliCalculateBlockSizeAutoType`
*   **C Signature**: `double ZopfliCalculateBlockSizeAutoType(const ZopfliLZ77Store* lz77, size_t lstart, size_t lend)`
*   **`lz77`, `lstart`, `lend`**: As above.
*   **Return Value**: `double`. Estimated compressed size in bits, choosing the best `btype`.
*   **Purpose**: Estimates block size by checking all three `btype` options.
*   **Porting Issues**: Floating point return.

### Function: `ZopfliDeflatePart`
*   **C Signature**: `void ZopfliDeflatePart(const ZopfliOptions* options, int btype, int final, const unsigned char* in, size_t instart, size_t inend, unsigned char* bp, unsigned char** out, size_t* outsize)`
*   **`options`**: Non-nullable pointer to `ZopfliOptions`.
*   **`btype`**: `int`. Target DEFLATE block type. 0=Store, 1=Fixed, 2=Dynamic. Special: if `btype` is not 0 or 1, it's treated as "auto" type selection per block.
*   **`final`**: `int`. True if this is the final DEFLATE block.
*   **`in`, `instart`, `inend`**: Input data and range.
*   **`bp`**: Non-nullable pointer to current bit pointer (0-7) for `*out`.
*   **`out`**: Non-nullable pointer to `unsigned char*`. Output buffer for DEFLATE stream. Grown by `ZOPFLI_APPEND_DATA`.
*   **`outsize`**: Non-nullable pointer to `size_t`. Current size of `*out`.
*   **Purpose**: Compresses a part of the input data (`in[instart..inend)`) into DEFLATE format.
    *   If `btype` is 0 (stored) or 1 (fixed), it processes the whole part as one block. For `btype` 1, it uses `ZopfliLZ77OptimalFixed`.
    *   If `btype` is 2 (dynamic, or auto), it may perform block splitting (`ZopfliBlockSplit`). For each sub-block, it runs `ZopfliLZ77Optimal` and then `AddLZ77BlockAutoType`.
*   **Side Effects**: Modifies `*bp`, `*out`, `*outsize`. Allocates temporary memory.
*   **Porting Issues**: High-level function orchestrating many parts. `ZOPFLI_APPEND_DATA`. Bit stream handling.

### Function: `ZopfliDeflate`
*   **C Signature**: `void ZopfliDeflate(const ZopfliOptions* options, int btype, int final, const unsigned char* in, size_t insize, unsigned char* bp, unsigned char** out, size_t* outsize)`
*   **`options`, `btype`, `final`, `in`, `insize`, `bp`, `out`, `outsize`**: Similar to `ZopfliDeflatePart`. `insize` is total input size.
*   **Purpose**: Main DEFLATE compression function. If `ZOPFLI_MASTER_BLOCK_SIZE` is non-zero and less than `insize`, it splits the input into "master blocks" of this size and calls `ZopfliDeflatePart` for each. Otherwise, calls `ZopfliDeflatePart` once for the whole input.
*   **Side Effects**: Modifies `*bp`, `*out`, `*outsize`.
*   **Porting Issues**: `ZOPFLI_MASTER_BLOCK_SIZE` logic. Verbose output.

## File: src/zopfli/gzip_container.h (and gzip_container.c)

### Static Helper Function: `CRC` (in gzip_container.c)
*   **C Signature**: `static unsigned long CRC(const unsigned char* data, size_t size)`
*   **Purpose**: Calculates CRC32 checksum. Uses `crc32_table`.
*   **Porting Issues**: `crc32_table` needs to be included. Standard CRC32 algorithms exist in Rust crates (e.g., `crc32fast`).

### Function: `ZopfliGzipCompress`
*   **C Signature**: `void ZopfliGzipCompress(const ZopfliOptions* options, const unsigned char* in, size_t insize, unsigned char** out, size_t* outsize)`
*   **`options`**: Non-nullable pointer to `ZopfliOptions`.
*   **`in`**: Non-nullable pointer to input data.
*   **`insize`**: `size_t`. Size of input data.
*   **`out`, `outsize`**: Output Gzip stream.
*   **Purpose**: Compresses data into Gzip format.
    1.  Writes Gzip header.
    2.  Calls `ZopfliDeflate` (with `btype=2` (dynamic/auto), `final=1`).
    3.  Appends CRC32 checksum and original input size (ISIZE) as Gzip trailer.
*   **Side Effects**: Modifies `*out`, `*outsize`.
*   **Porting Issues**: `ZOPFLI_APPEND_DATA`. Gzip header/trailer format details. CRC32 calculation.

## File: src/zopfli/zlib_container.h (and zlib_container.c)

### Static Helper Function: `adler32` (in zlib_container.c)
*   **C Signature**: `static unsigned adler32(const unsigned char* data, size_t size)`
*   **Purpose**: Calculates Adler-32 checksum.
*   **Porting Issues**: Standard Adler-32 algorithms exist in Rust crates (e.g. `adler32`).

### Function: `ZopfliZlibCompress`
*   **C Signature**: `void ZopfliZlibCompress(const ZopfliOptions* options, const unsigned char* in, size_t insize, unsigned char** out, size_t* outsize)`
*   **Arguments**: Same as `ZopfliGzipCompress`.
*   **`out`, `outsize`**: Output Zlib stream.
*   **Purpose**: Compresses data into Zlib format.
    1.  Writes Zlib header (CMF, FLG bytes).
    2.  Calls `ZopfliDeflate` (with `btype=2`, `final=1`).
    3.  Appends Adler-32 checksum as Zlib trailer.
*   **Side Effects**: Modifies `*out`, `*outsize`.
*   **Porting Issues**: `ZOPFLI_APPEND_DATA`. Zlib header/trailer format details. Adler-32 calculation.

## File: src/zopfli/zopfli_lib.c

### Function: `ZopfliCompress`
*   **C Signature**: `void ZopfliCompress(const ZopfliOptions* options, ZopfliFormat output_type, const unsigned char* in, size_t insize, unsigned char** out, size_t* outsize)`
*   **`options`**: Non-nullable pointer to `ZopfliOptions`.
*   **`output_type`**: `ZopfliFormat` enum specifying Gzip, Zlib, or raw Deflate.
*   **`in`, `insize`**: Input data.
*   **`out`, `outsize`**: Output compressed stream.
*   **Purpose**: Top-level compression entry point. Dispatches to `ZopfliGzipCompress`, `ZopfliZlibCompress`, or `ZopfliDeflate` based on `output_type`.
*   **Side Effects**: Modifies `*out`, `*outsize`. `assert(0)` for unknown `output_type`.
*   **Rust Equivalent**: The main public API function for the library.
*   **Porting Issues**: `assert(0)` should be an error return.

## File: src/zopflipng/zopflipng_lib.h
This file defines the C API for ZopfliPNG, which uses the Zopfli library for DEFLATE compression within PNG files.

### Enum: `ZopfliPNGFilterStrategy`
*   **C Definition**:
    ```c
    enum ZopfliPNGFilterStrategy {
      kStrategyZero = 0, /* ... */ kNumFilterStrategies
    };
    ```
*   **Purpose**: Specifies PNG filter strategy options for ZopfliPNG.
*   **Rust Equivalent**: `pub enum ZopfliPNGFilterStrategy { ... }`.
*   **Porting Issues**: None.

### Structure: `CZopfliPNGOptions`
*   **C Definition**:
    ```c
    typedef struct CZopfliPNGOptions {
      int lossy_transparent;
      int lossy_8bit;
      enum ZopfliPNGFilterStrategy* filter_strategies; // Array
      int num_filter_strategies;
      int auto_filter_strategy;
      char** keepchunks; // Array of C strings
      int num_keepchunks;
      int use_zopfli; // If 0, uses libdeflate or Zlib's deflate
      int num_iterations; // Zopfli iterations
      int num_iterations_large; // Zopfli iterations for large files
      int block_split_strategy; // Zopfli block splitting options
    } CZopfliPNGOptions;
    ```
*   **Members**: Various options for PNG optimization. `filter_strategies` and `keepchunks` are dynamically allocated arrays.
*   **Purpose**: Configuration for `CZopfliPNGOptimize`.
*   **Rust Equivalent**:
    ```rust
    pub struct CZopfliPNGOptions {
        pub lossy_transparent: bool,
        pub lossy_8bit: bool,
        pub filter_strategies: Vec<ZopfliPNGFilterStrategy>,
        // num_filter_strategies is Vec::len()
        pub auto_filter_strategy: bool,
        pub keep_chunks: Vec<String>, // Or Vec<CString> if interop is key
        // num_keepchunks is Vec::len()
        pub use_zopfli: bool,
        pub num_iterations: i32,
        pub num_iterations_large: i32,
        pub block_split_strategy: i32, // Map to ZopfliOptions equivalent
    }
    ```
*   **Porting Issues**: Management of `filter_strategies` and `keepchunks` arrays. Mapping `block_split_strategy` to Zopfli core options. The C++ version in the same file (`ZopfliPNGOptions`) shows a more idiomatic C++/Rust way with `std::vector` and `std::string`.

### Function: `CZopfliPNGSetDefaults`
*   **C Signature**: `void CZopfliPNGSetDefaults(CZopfliPNGOptions *png_options)`
*   **`png_options`**: Non-nullable pointer to `CZopfliPNGOptions` to initialize.
*   **Purpose**: Initializes `CZopfliPNGOptions` with default values.
*   **Side Effects**: Modifies `png_options`. May allocate default `filter_strategies`.
*   **Rust Equivalent**: `impl Default for CZopfliPNGOptions`.
*   **Porting Issues**: Default allocation for `filter_strategies`.

### Function: `CZopfliPNGOptimize`
*   **C Signature**: `int CZopfliPNGOptimize(const unsigned char* origpng, const size_t origpng_size, const CZopfliPNGOptions* png_options, int verbose, unsigned char** resultpng, size_t* resultpng_size)`
*   **`origpng`, `origpng_size`**: Input PNG data.
*   **`png_options`**: Non-nullable pointer to ZopfliPNG options.
*   **`verbose`**: `int`. Verbosity flag.
*   **`resultpng`, `resultpng_size`**: Output optimized PNG data.
*   **Return Value**: `int`. Error code (0 for success).
*   **Purpose**: Optimizes a PNG file using Zopfli for IDAT chunk compression and other PNG-specific strategies. This function would call into the Zopfli core library (e.g., `ZopfliDeflate` or similar) after decoding PNG structure with LodePNG.
*   **Rust Equivalent**: A high-level function, likely returning `Result<Vec<u8>, PngError>`.
*   **Porting Issues**: This is a complex function that involves:
    *   PNG parsing (likely using LodePNG C functions internally, or a Rust PNG library).
    *   Applying various PNG optimization strategies.
    *   Re-compressing IDAT chunks using Zopfli.
    *   Re-assembling the PNG.
    The focus of the port is "this C library [Zopfli]", so the internals of `CZopfliPNGOptimize` related to PNG structure are less critical than how it *uses* Zopfli. The `png_options` map to `ZopfliOptions` for the core compression part.

## File: src/zopflipng/lodepng/lodepng.h
This is a third-party library for PNG encoding/decoding. A full port of LodePNG is outside the scope of porting Zopfli. However, if ZopfliPNG uses specific LodePNG C structs or functions that are passed to/from Zopfli core or are fundamental to ZopfliPNG's C API, they would be relevant.
The provided `lodepng.h` contains many declarations. Key items used by a ZopfliPNG-like tool would be:
*   `LodePNGColorType`: Enum for PNG color types.
*   `LodePNGState`: A large struct holding decoder/encoder settings, color mode information, PNG info chunks, and error state.
*   `lodepng_decode()` / `lodepng_encode()`: High-level functions using `LodePNGState`.
*   Chunk handling functions: `lodepng_chunk_find`, `lodepng_chunk_data`, etc.
*   Zlib/Deflate functions: `lodepng_deflate`, `lodepng_inflate`. ZopfliPNG replaces LodePNG's `custom_deflate` with Zopfli's.

**Porting LodePNG-dependent parts (like ZopfliPNG):**
A Rust port of ZopfliPNG would typically use a Rust PNG library (e.g., `png`) instead of porting LodePNG or calling its C version. The `CZopfliPNGOptions` would be translated to configure both the Rust PNG library and the ported Zopfli Rust library.

## File: src/zopflipng/lodepng/lodepng_util.h
This file contains C++ utility functions for LodePNG, mostly using `std::vector` and `std::string`. These are not part of the Zopfli C library itself and are specific to the ZopfliPNG C++ tool. The C API of ZopfliPNG (`zopflipng_lib.h`) is what matters for C library interoperability. The functions like `getChunkInfo`, `getChunks` are C++ based.

The struct `ZlibBlockInfo` is defined here, but it uses `std::vector`, so it's C++ specific. It's for analyzing Zlib streams, not direct compression.

## Main function (`zopfli_bin.c`)
The `main` function in `zopfli_bin.c` demonstrates how `ZopfliOptions` are set from command-line arguments and how `ZopfliCompress` is called.
Helper functions `LoadFile` and `SaveFile` are for I/O.
*   **Porting Issues**: `LoadFile`/`SaveFile` map to Rust's `std::fs::read` and `std::fs::write`. Command-line parsing would use a crate like `clap`. The string manipulation (`AddStrings`) is for filename generation.

This covers the main structures and functions. The port to Rust will involve careful handling of memory, pointers, and error codes, leveraging Rust's safety features and standard library/crates. The core algorithms (LZ77, Huffman coding, dynamic programming in squeeze) will be the most challenging to translate correctly and efficiently.