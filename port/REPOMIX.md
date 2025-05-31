This file is a merged representation of a subset of the codebase, containing specifically included files, combined into a single document by Repomix.
The content has been processed where comments have been removed, empty lines have been removed.

# File Summary

## Purpose
This file contains a packed representation of the entire repository's contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.

## File Format
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Repository files (if enabled)
5. Multiple file entries, each consisting of:
  a. A header with the file path (## File: path/to/file)
  b. The full contents of the file in a code block

## Usage Guidelines
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.

## Notes
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Only files matching these patterns are included: **/*.c, **/*.h
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded
- Code comments have been removed from supported file types
- Empty lines have been removed from all files
- Files are sorted by Git change count (files with more changes are at the bottom)

# Directory Structure
```
src/
  zopfli/
    blocksplitter.c
    blocksplitter.h
    cache.c
    cache.h
    deflate.c
    deflate.h
    gzip_container.c
    gzip_container.h
    hash.c
    hash.h
    katajainen.c
    katajainen.h
    lz77.c
    lz77.h
    squeeze.c
    squeeze.h
    symbols.h
    tree.c
    tree.h
    util.c
    util.h
    zlib_container.c
    zlib_container.h
    zopfli_bin.c
    zopfli_lib.c
    zopfli.h
  zopflipng/
    lodepng/
      lodepng_util.h
      lodepng.h
    zopflipng_lib.h
```

# Files

## File: src/zopfli/cache.h
```
#ifndef ZOPFLI_CACHE_H_
#define ZOPFLI_CACHE_H_
#include "util.h"
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
typedef struct ZopfliLongestMatchCache {
  unsigned short* length;
  unsigned short* dist;
  unsigned char* sublen;
} ZopfliLongestMatchCache;
void ZopfliInitCache(size_t blocksize, ZopfliLongestMatchCache* lmc);
void ZopfliCleanCache(ZopfliLongestMatchCache* lmc);
void ZopfliSublenToCache(const unsigned short* sublen,
                         size_t pos, size_t length,
                         ZopfliLongestMatchCache* lmc);
void ZopfliCacheToSublen(const ZopfliLongestMatchCache* lmc,
                         size_t pos, size_t length,
                         unsigned short* sublen);
unsigned ZopfliMaxCachedSublen(const ZopfliLongestMatchCache* lmc,
                               size_t pos, size_t length);
#endif
#endif
```

## File: src/zopfli/gzip_container.h
```
#ifndef ZOPFLI_GZIP_H_
#define ZOPFLI_GZIP_H_
#include "zopfli.h"
#ifdef __cplusplus
extern "C" {
#endif
void ZopfliGzipCompress(const ZopfliOptions* options,
                        const unsigned char* in, size_t insize,
                        unsigned char** out, size_t* outsize);
#ifdef __cplusplus
}
#endif
#endif
```

## File: src/zopfli/tree.c
```cpp
#include "tree.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include "katajainen.h"
#include "util.h"
void ZopfliLengthsToSymbols(const unsigned* lengths, size_t n, unsigned maxbits,
                            unsigned* symbols) {
  size_t* bl_count = (size_t*)malloc(sizeof(size_t) * (maxbits + 1));
  size_t* next_code = (size_t*)malloc(sizeof(size_t) * (maxbits + 1));
  unsigned bits, i;
  unsigned code;
  for (i = 0; i < n; i++) {
    symbols[i] = 0;
  }
  for (bits = 0; bits <= maxbits; bits++) {
    bl_count[bits] = 0;
  }
  for (i = 0; i < n; i++) {
    assert(lengths[i] <= maxbits);
    bl_count[lengths[i]]++;
  }
  code = 0;
  bl_count[0] = 0;
  for (bits = 1; bits <= maxbits; bits++) {
    code = (code + bl_count[bits-1]) << 1;
    next_code[bits] = code;
  }
  for (i = 0;  i < n; i++) {
    unsigned len = lengths[i];
    if (len != 0) {
      symbols[i] = next_code[len];
      next_code[len]++;
    }
  }
  free(bl_count);
  free(next_code);
}
void ZopfliCalculateEntropy(const size_t* count, size_t n, double* bitlengths) {
  static const double kInvLog2 = 1.4426950408889;
  unsigned sum = 0;
  unsigned i;
  double log2sum;
  for (i = 0; i < n; ++i) {
    sum += count[i];
  }
  log2sum = (sum == 0 ? log(n) : log(sum)) * kInvLog2;
  for (i = 0; i < n; ++i) {
    if (count[i] == 0) bitlengths[i] = log2sum;
    else bitlengths[i] = log2sum - log(count[i]) * kInvLog2;
    if (bitlengths[i] < 0 && bitlengths[i] > -1e-5) bitlengths[i] = 0;
    assert(bitlengths[i] >= 0);
  }
}
void ZopfliCalculateBitLengths(const size_t* count, size_t n, int maxbits,
                               unsigned* bitlengths) {
  int error = ZopfliLengthLimitedCodeLengths(count, n, maxbits, bitlengths);
  (void) error;
  assert(!error);
}
```

## File: src/zopfli/tree.h
```
#ifndef ZOPFLI_TREE_H_
#define ZOPFLI_TREE_H_
#include <string.h>
void ZopfliCalculateBitLengths(const size_t* count, size_t n, int maxbits,
                               unsigned *bitlengths);
void ZopfliLengthsToSymbols(const unsigned* lengths, size_t n, unsigned maxbits,
                            unsigned* symbols);
void ZopfliCalculateEntropy(const size_t* count, size_t n, double* bitlengths);
#endif
```

## File: src/zopfli/zlib_container.h
```
#ifndef ZOPFLI_ZLIB_H_
#define ZOPFLI_ZLIB_H_
#include "zopfli.h"
#ifdef __cplusplus
extern "C" {
#endif
void ZopfliZlibCompress(const ZopfliOptions* options,
                        const unsigned char* in, size_t insize,
                        unsigned char** out, size_t* outsize);
#ifdef __cplusplus
}
#endif
#endif
```

## File: src/zopfli/zopfli_lib.c
```cpp
#include "zopfli.h"
#include "deflate.h"
#include "gzip_container.h"
#include "zlib_container.h"
#include <assert.h>
void ZopfliCompress(const ZopfliOptions* options, ZopfliFormat output_type,
                    const unsigned char* in, size_t insize,
                    unsigned char** out, size_t* outsize) {
  if (output_type == ZOPFLI_FORMAT_GZIP) {
    ZopfliGzipCompress(options, in, insize, out, outsize);
  } else if (output_type == ZOPFLI_FORMAT_ZLIB) {
    ZopfliZlibCompress(options, in, insize, out, outsize);
  } else if (output_type == ZOPFLI_FORMAT_DEFLATE) {
    unsigned char bp = 0;
    ZopfliDeflate(options, 2 , 1,
                  in, insize, &bp, out, outsize);
  } else {
    assert(0);
  }
}
```

## File: src/zopfli/blocksplitter.h
```
#ifndef ZOPFLI_BLOCKSPLITTER_H_
#define ZOPFLI_BLOCKSPLITTER_H_
#include <stdlib.h>
#include "lz77.h"
#include "zopfli.h"
void ZopfliBlockSplitLZ77(const ZopfliOptions* options,
                          const ZopfliLZ77Store* lz77, size_t maxblocks,
                          size_t** splitpoints, size_t* npoints);
void ZopfliBlockSplit(const ZopfliOptions* options,
                      const unsigned char* in, size_t instart, size_t inend,
                      size_t maxblocks, size_t** splitpoints, size_t* npoints);
void ZopfliBlockSplitSimple(const unsigned char* in,
                            size_t instart, size_t inend,
                            size_t blocksize,
                            size_t** splitpoints, size_t* npoints);
#endif
```

## File: src/zopfli/deflate.h
```
#ifndef ZOPFLI_DEFLATE_H_
#define ZOPFLI_DEFLATE_H_
#include "lz77.h"
#include "zopfli.h"
#ifdef __cplusplus
extern "C" {
#endif
void ZopfliDeflate(const ZopfliOptions* options, int btype, int final,
                   const unsigned char* in, size_t insize,
                   unsigned char* bp, unsigned char** out, size_t* outsize);
void ZopfliDeflatePart(const ZopfliOptions* options, int btype, int final,
                       const unsigned char* in, size_t instart, size_t inend,
                       unsigned char* bp, unsigned char** out,
                       size_t* outsize);
double ZopfliCalculateBlockSize(const ZopfliLZ77Store* lz77,
                                size_t lstart, size_t lend, int btype);
double ZopfliCalculateBlockSizeAutoType(const ZopfliLZ77Store* lz77,
                                        size_t lstart, size_t lend);
#ifdef __cplusplus
}
#endif
#endif
```

## File: src/zopfli/katajainen.h
```
#ifndef ZOPFLI_KATAJAINEN_H_
#define ZOPFLI_KATAJAINEN_H_
#include <string.h>
int ZopfliLengthLimitedCodeLengths(
    const size_t* frequencies, int n, int maxbits, unsigned* bitlengths);
#endif
```

## File: src/zopfli/squeeze.h
```
#ifndef ZOPFLI_SQUEEZE_H_
#define ZOPFLI_SQUEEZE_H_
#include "lz77.h"
void ZopfliLZ77Optimal(ZopfliBlockState *s,
                       const unsigned char* in, size_t instart, size_t inend,
                       int numiterations,
                       ZopfliLZ77Store* store);
void ZopfliLZ77OptimalFixed(ZopfliBlockState *s,
                            const unsigned char* in,
                            size_t instart, size_t inend,
                            ZopfliLZ77Store* store);
#endif
```

## File: src/zopfli/symbols.h
```
#ifndef ZOPFLI_SYMBOLS_H_
#define ZOPFLI_SYMBOLS_H_
#ifdef __has_builtin
# if __has_builtin(__builtin_clz)
#   define ZOPFLI_HAS_BUILTIN_CLZ
# endif
#elif __GNUC__ * 100 + __GNUC_MINOR__ >= 304
# define ZOPFLI_HAS_BUILTIN_CLZ
#endif
static int ZopfliGetDistExtraBits(int dist) {
#ifdef ZOPFLI_HAS_BUILTIN_CLZ
  if (dist < 5) return 0;
  return (31 ^ __builtin_clz(dist - 1)) - 1;
#else
  if (dist < 5) return 0;
  else if (dist < 9) return 1;
  else if (dist < 17) return 2;
  else if (dist < 33) return 3;
  else if (dist < 65) return 4;
  else if (dist < 129) return 5;
  else if (dist < 257) return 6;
  else if (dist < 513) return 7;
  else if (dist < 1025) return 8;
  else if (dist < 2049) return 9;
  else if (dist < 4097) return 10;
  else if (dist < 8193) return 11;
  else if (dist < 16385) return 12;
  else return 13;
#endif
}
static int ZopfliGetDistExtraBitsValue(int dist) {
#ifdef ZOPFLI_HAS_BUILTIN_CLZ
  if (dist < 5) {
    return 0;
  } else {
    int l = 31 ^ __builtin_clz(dist - 1);
    return (dist - (1 + (1 << l))) & ((1 << (l - 1)) - 1);
  }
#else
  if (dist < 5) return 0;
  else if (dist < 9) return (dist - 5) & 1;
  else if (dist < 17) return (dist - 9) & 3;
  else if (dist < 33) return (dist - 17) & 7;
  else if (dist < 65) return (dist - 33) & 15;
  else if (dist < 129) return (dist - 65) & 31;
  else if (dist < 257) return (dist - 129) & 63;
  else if (dist < 513) return (dist - 257) & 127;
  else if (dist < 1025) return (dist - 513) & 255;
  else if (dist < 2049) return (dist - 1025) & 511;
  else if (dist < 4097) return (dist - 2049) & 1023;
  else if (dist < 8193) return (dist - 4097) & 2047;
  else if (dist < 16385) return (dist - 8193) & 4095;
  else return (dist - 16385) & 8191;
#endif
}
static int ZopfliGetDistSymbol(int dist) {
#ifdef ZOPFLI_HAS_BUILTIN_CLZ
  if (dist < 5) {
    return dist - 1;
  } else {
    int l = (31 ^ __builtin_clz(dist - 1));
    int r = ((dist - 1) >> (l - 1)) & 1;
    return l * 2 + r;
  }
#else
  if (dist < 193) {
    if (dist < 13) {
      if (dist < 5) return dist - 1;
      else if (dist < 7) return 4;
      else if (dist < 9) return 5;
      else return 6;
    } else {
      if (dist < 17) return 7;
      else if (dist < 25) return 8;
      else if (dist < 33) return 9;
      else if (dist < 49) return 10;
      else if (dist < 65) return 11;
      else if (dist < 97) return 12;
      else if (dist < 129) return 13;
      else return 14;
    }
  } else {
    if (dist < 2049) {
      if (dist < 257) return 15;
      else if (dist < 385) return 16;
      else if (dist < 513) return 17;
      else if (dist < 769) return 18;
      else if (dist < 1025) return 19;
      else if (dist < 1537) return 20;
      else return 21;
    } else {
      if (dist < 3073) return 22;
      else if (dist < 4097) return 23;
      else if (dist < 6145) return 24;
      else if (dist < 8193) return 25;
      else if (dist < 12289) return 26;
      else if (dist < 16385) return 27;
      else if (dist < 24577) return 28;
      else return 29;
    }
  }
#endif
}
static int ZopfliGetLengthExtraBits(int l) {
  static const int table[259] = {
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0
  };
  return table[l];
}
static int ZopfliGetLengthExtraBitsValue(int l) {
  static const int table[259] = {
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 2, 3, 0,
    1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
    6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6,
    7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2,
    3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 0, 1, 2, 3, 4, 5, 6,
    7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 0
  };
  return table[l];
}
static int ZopfliGetLengthSymbol(int l) {
  static const int table[259] = {
    0, 0, 0, 257, 258, 259, 260, 261, 262, 263, 264,
    265, 265, 266, 266, 267, 267, 268, 268,
    269, 269, 269, 269, 270, 270, 270, 270,
    271, 271, 271, 271, 272, 272, 272, 272,
    273, 273, 273, 273, 273, 273, 273, 273,
    274, 274, 274, 274, 274, 274, 274, 274,
    275, 275, 275, 275, 275, 275, 275, 275,
    276, 276, 276, 276, 276, 276, 276, 276,
    277, 277, 277, 277, 277, 277, 277, 277,
    277, 277, 277, 277, 277, 277, 277, 277,
    278, 278, 278, 278, 278, 278, 278, 278,
    278, 278, 278, 278, 278, 278, 278, 278,
    279, 279, 279, 279, 279, 279, 279, 279,
    279, 279, 279, 279, 279, 279, 279, 279,
    280, 280, 280, 280, 280, 280, 280, 280,
    280, 280, 280, 280, 280, 280, 280, 280,
    281, 281, 281, 281, 281, 281, 281, 281,
    281, 281, 281, 281, 281, 281, 281, 281,
    281, 281, 281, 281, 281, 281, 281, 281,
    281, 281, 281, 281, 281, 281, 281, 281,
    282, 282, 282, 282, 282, 282, 282, 282,
    282, 282, 282, 282, 282, 282, 282, 282,
    282, 282, 282, 282, 282, 282, 282, 282,
    282, 282, 282, 282, 282, 282, 282, 282,
    283, 283, 283, 283, 283, 283, 283, 283,
    283, 283, 283, 283, 283, 283, 283, 283,
    283, 283, 283, 283, 283, 283, 283, 283,
    283, 283, 283, 283, 283, 283, 283, 283,
    284, 284, 284, 284, 284, 284, 284, 284,
    284, 284, 284, 284, 284, 284, 284, 284,
    284, 284, 284, 284, 284, 284, 284, 284,
    284, 284, 284, 284, 284, 284, 284, 285
  };
  return table[l];
}
static int ZopfliGetLengthSymbolExtraBits(int s) {
  static const int table[29] = {
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
    3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
  };
  return table[s - 257];
}
static int ZopfliGetDistSymbolExtraBits(int s) {
  static const int table[30] = {
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
    9, 9, 10, 10, 11, 11, 12, 12, 13, 13
  };
  return table[s];
}
#endif
```

## File: src/zopfli/zlib_container.c
```cpp
#include "zlib_container.h"
#include "util.h"
#include <stdio.h>
#include "deflate.h"
static unsigned adler32(const unsigned char* data, size_t size)
{
  static const unsigned sums_overflow = 5550;
  unsigned s1 = 1;
  unsigned s2 = 1 >> 16;
  while (size > 0) {
    size_t amount = size > sums_overflow ? sums_overflow : size;
    size -= amount;
    while (amount > 0) {
      s1 += (*data++);
      s2 += s1;
      amount--;
    }
    s1 %= 65521;
    s2 %= 65521;
  }
  return (s2 << 16) | s1;
}
void ZopfliZlibCompress(const ZopfliOptions* options,
                        const unsigned char* in, size_t insize,
                        unsigned char** out, size_t* outsize) {
  unsigned char bitpointer = 0;
  unsigned checksum = adler32(in, (unsigned)insize);
  unsigned cmf = 120;
  unsigned flevel = 3;
  unsigned fdict = 0;
  unsigned cmfflg = 256 * cmf + fdict * 32 + flevel * 64;
  unsigned fcheck = 31 - cmfflg % 31;
  cmfflg += fcheck;
  ZOPFLI_APPEND_DATA(cmfflg / 256, out, outsize);
  ZOPFLI_APPEND_DATA(cmfflg % 256, out, outsize);
  ZopfliDeflate(options, 2 , 1 ,
                in, insize, &bitpointer, out, outsize);
  ZOPFLI_APPEND_DATA((checksum >> 24) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((checksum >> 16) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((checksum >> 8) % 256, out, outsize);
  ZOPFLI_APPEND_DATA(checksum % 256, out, outsize);
  if (options->verbose) {
    fprintf(stderr,
            "Original Size: %d, Zlib: %d, Compression: %f%% Removed\n",
            (int)insize, (int)*outsize,
            100.0 * (double)(insize - *outsize) / (double)insize);
  }
}
```

## File: src/zopfli/zopfli.h
```
#ifndef ZOPFLI_ZOPFLI_H_
#define ZOPFLI_ZOPFLI_H_
#include <stddef.h>
#include <stdlib.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct ZopfliOptions {
  int verbose;
  int verbose_more;
  int numiterations;
  int blocksplitting;
  int blocksplittinglast;
  int blocksplittingmax;
} ZopfliOptions;
void ZopfliInitOptions(ZopfliOptions* options);
typedef enum {
  ZOPFLI_FORMAT_GZIP,
  ZOPFLI_FORMAT_ZLIB,
  ZOPFLI_FORMAT_DEFLATE
} ZopfliFormat;
void ZopfliCompress(const ZopfliOptions* options, ZopfliFormat output_type,
                    const unsigned char* in, size_t insize,
                    unsigned char** out, size_t* outsize);
#ifdef __cplusplus
}
#endif
#endif
```

## File: src/zopfli/blocksplitter.c
```cpp
#include "blocksplitter.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include "deflate.h"
#include "squeeze.h"
#include "tree.h"
#include "util.h"
typedef double FindMinimumFun(size_t i, void* context);
static size_t FindMinimum(FindMinimumFun f, void* context,
                          size_t start, size_t end, double* smallest) {
  if (end - start < 1024) {
    double best = ZOPFLI_LARGE_FLOAT;
    size_t result = start;
    size_t i;
    for (i = start; i < end; i++) {
      double v = f(i, context);
      if (v < best) {
        best = v;
        result = i;
      }
    }
    *smallest = best;
    return result;
  } else {
#define NUM 9
    size_t i;
    size_t p[NUM];
    double vp[NUM];
    size_t besti;
    double best;
    double lastbest = ZOPFLI_LARGE_FLOAT;
    size_t pos = start;
    for (;;) {
      if (end - start <= NUM) break;
      for (i = 0; i < NUM; i++) {
        p[i] = start + (i + 1) * ((end - start) / (NUM + 1));
        vp[i] = f(p[i], context);
      }
      besti = 0;
      best = vp[0];
      for (i = 1; i < NUM; i++) {
        if (vp[i] < best) {
          best = vp[i];
          besti = i;
        }
      }
      if (best > lastbest) break;
      start = besti == 0 ? start : p[besti - 1];
      end = besti == NUM - 1 ? end : p[besti + 1];
      pos = p[besti];
      lastbest = best;
    }
    *smallest = lastbest;
    return pos;
#undef NUM
  }
}
static double EstimateCost(const ZopfliLZ77Store* lz77,
                           size_t lstart, size_t lend) {
  return ZopfliCalculateBlockSizeAutoType(lz77, lstart, lend);
}
typedef struct SplitCostContext {
  const ZopfliLZ77Store* lz77;
  size_t start;
  size_t end;
} SplitCostContext;
static double SplitCost(size_t i, void* context) {
  SplitCostContext* c = (SplitCostContext*)context;
  return EstimateCost(c->lz77, c->start, i) + EstimateCost(c->lz77, i, c->end);
}
static void AddSorted(size_t value, size_t** out, size_t* outsize) {
  size_t i;
  ZOPFLI_APPEND_DATA(value, out, outsize);
  for (i = 0; i + 1 < *outsize; i++) {
    if ((*out)[i] > value) {
      size_t j;
      for (j = *outsize - 1; j > i; j--) {
        (*out)[j] = (*out)[j - 1];
      }
      (*out)[i] = value;
      break;
    }
  }
}
static void PrintBlockSplitPoints(const ZopfliLZ77Store* lz77,
                                  const size_t* lz77splitpoints,
                                  size_t nlz77points) {
  size_t* splitpoints = 0;
  size_t npoints = 0;
  size_t i;
  size_t pos = 0;
  if (nlz77points > 0) {
    for (i = 0; i < lz77->size; i++) {
      size_t length = lz77->dists[i] == 0 ? 1 : lz77->litlens[i];
      if (lz77splitpoints[npoints] == i) {
        ZOPFLI_APPEND_DATA(pos, &splitpoints, &npoints);
        if (npoints == nlz77points) break;
      }
      pos += length;
    }
  }
  assert(npoints == nlz77points);
  fprintf(stderr, "block split points: ");
  for (i = 0; i < npoints; i++) {
    fprintf(stderr, "%d ", (int)splitpoints[i]);
  }
  fprintf(stderr, "(hex:");
  for (i = 0; i < npoints; i++) {
    fprintf(stderr, " %x", (int)splitpoints[i]);
  }
  fprintf(stderr, ")\n");
  free(splitpoints);
}
static int FindLargestSplittableBlock(
    size_t lz77size, const unsigned char* done,
    const size_t* splitpoints, size_t npoints,
    size_t* lstart, size_t* lend) {
  size_t longest = 0;
  int found = 0;
  size_t i;
  for (i = 0; i <= npoints; i++) {
    size_t start = i == 0 ? 0 : splitpoints[i - 1];
    size_t end = i == npoints ? lz77size - 1 : splitpoints[i];
    if (!done[start] && end - start > longest) {
      *lstart = start;
      *lend = end;
      found = 1;
      longest = end - start;
    }
  }
  return found;
}
void ZopfliBlockSplitLZ77(const ZopfliOptions* options,
                          const ZopfliLZ77Store* lz77, size_t maxblocks,
                          size_t** splitpoints, size_t* npoints) {
  size_t lstart, lend;
  size_t i;
  size_t llpos = 0;
  size_t numblocks = 1;
  unsigned char* done;
  double splitcost, origcost;
  if (lz77->size < 10) return;
  done = (unsigned char*)malloc(lz77->size);
  if (!done) exit(-1);
  for (i = 0; i < lz77->size; i++) done[i] = 0;
  lstart = 0;
  lend = lz77->size;
  for (;;) {
    SplitCostContext c;
    if (maxblocks > 0 && numblocks >= maxblocks) {
      break;
    }
    c.lz77 = lz77;
    c.start = lstart;
    c.end = lend;
    assert(lstart < lend);
    llpos = FindMinimum(SplitCost, &c, lstart + 1, lend, &splitcost);
    assert(llpos > lstart);
    assert(llpos < lend);
    origcost = EstimateCost(lz77, lstart, lend);
    if (splitcost > origcost || llpos == lstart + 1 || llpos == lend) {
      done[lstart] = 1;
    } else {
      AddSorted(llpos, splitpoints, npoints);
      numblocks++;
    }
    if (!FindLargestSplittableBlock(
        lz77->size, done, *splitpoints, *npoints, &lstart, &lend)) {
      break;
    }
    if (lend - lstart < 10) {
      break;
    }
  }
  if (options->verbose) {
    PrintBlockSplitPoints(lz77, *splitpoints, *npoints);
  }
  free(done);
}
void ZopfliBlockSplit(const ZopfliOptions* options,
                      const unsigned char* in, size_t instart, size_t inend,
                      size_t maxblocks, size_t** splitpoints, size_t* npoints) {
  size_t pos = 0;
  size_t i;
  ZopfliBlockState s;
  size_t* lz77splitpoints = 0;
  size_t nlz77points = 0;
  ZopfliLZ77Store store;
  ZopfliHash hash;
  ZopfliHash* h = &hash;
  ZopfliInitLZ77Store(in, &store);
  ZopfliInitBlockState(options, instart, inend, 0, &s);
  ZopfliAllocHash(ZOPFLI_WINDOW_SIZE, h);
  *npoints = 0;
  *splitpoints = 0;
  ZopfliLZ77Greedy(&s, in, instart, inend, &store, h);
  ZopfliBlockSplitLZ77(options,
                       &store, maxblocks,
                       &lz77splitpoints, &nlz77points);
  pos = instart;
  if (nlz77points > 0) {
    for (i = 0; i < store.size; i++) {
      size_t length = store.dists[i] == 0 ? 1 : store.litlens[i];
      if (lz77splitpoints[*npoints] == i) {
        ZOPFLI_APPEND_DATA(pos, splitpoints, npoints);
        if (*npoints == nlz77points) break;
      }
      pos += length;
    }
  }
  assert(*npoints == nlz77points);
  free(lz77splitpoints);
  ZopfliCleanBlockState(&s);
  ZopfliCleanLZ77Store(&store);
  ZopfliCleanHash(h);
}
void ZopfliBlockSplitSimple(const unsigned char* in,
                            size_t instart, size_t inend,
                            size_t blocksize,
                            size_t** splitpoints, size_t* npoints) {
  size_t i = instart;
  while (i < inend) {
    ZOPFLI_APPEND_DATA(i, splitpoints, npoints);
    i += blocksize;
  }
  (void)in;
}
```

## File: src/zopfli/gzip_container.c
```cpp
#include "gzip_container.h"
#include "util.h"
#include <stdio.h>
#include "deflate.h"
static const unsigned long crc32_table[256] = {
           0u, 1996959894u, 3993919788u, 2567524794u,  124634137u, 1886057615u,
  3915621685u, 2657392035u,  249268274u, 2044508324u, 3772115230u, 2547177864u,
   162941995u, 2125561021u, 3887607047u, 2428444049u,  498536548u, 1789927666u,
  4089016648u, 2227061214u,  450548861u, 1843258603u, 4107580753u, 2211677639u,
   325883990u, 1684777152u, 4251122042u, 2321926636u,  335633487u, 1661365465u,
  4195302755u, 2366115317u,  997073096u, 1281953886u, 3579855332u, 2724688242u,
  1006888145u, 1258607687u, 3524101629u, 2768942443u,  901097722u, 1119000684u,
  3686517206u, 2898065728u,  853044451u, 1172266101u, 3705015759u, 2882616665u,
   651767980u, 1373503546u, 3369554304u, 3218104598u,  565507253u, 1454621731u,
  3485111705u, 3099436303u,  671266974u, 1594198024u, 3322730930u, 2970347812u,
   795835527u, 1483230225u, 3244367275u, 3060149565u, 1994146192u,   31158534u,
  2563907772u, 4023717930u, 1907459465u,  112637215u, 2680153253u, 3904427059u,
  2013776290u,  251722036u, 2517215374u, 3775830040u, 2137656763u,  141376813u,
  2439277719u, 3865271297u, 1802195444u,  476864866u, 2238001368u, 4066508878u,
  1812370925u,  453092731u, 2181625025u, 4111451223u, 1706088902u,  314042704u,
  2344532202u, 4240017532u, 1658658271u,  366619977u, 2362670323u, 4224994405u,
  1303535960u,  984961486u, 2747007092u, 3569037538u, 1256170817u, 1037604311u,
  2765210733u, 3554079995u, 1131014506u,  879679996u, 2909243462u, 3663771856u,
  1141124467u,  855842277u, 2852801631u, 3708648649u, 1342533948u,  654459306u,
  3188396048u, 3373015174u, 1466479909u,  544179635u, 3110523913u, 3462522015u,
  1591671054u,  702138776u, 2966460450u, 3352799412u, 1504918807u,  783551873u,
  3082640443u, 3233442989u, 3988292384u, 2596254646u,   62317068u, 1957810842u,
  3939845945u, 2647816111u,   81470997u, 1943803523u, 3814918930u, 2489596804u,
   225274430u, 2053790376u, 3826175755u, 2466906013u,  167816743u, 2097651377u,
  4027552580u, 2265490386u,  503444072u, 1762050814u, 4150417245u, 2154129355u,
   426522225u, 1852507879u, 4275313526u, 2312317920u,  282753626u, 1742555852u,
  4189708143u, 2394877945u,  397917763u, 1622183637u, 3604390888u, 2714866558u,
   953729732u, 1340076626u, 3518719985u, 2797360999u, 1068828381u, 1219638859u,
  3624741850u, 2936675148u,  906185462u, 1090812512u, 3747672003u, 2825379669u,
   829329135u, 1181335161u, 3412177804u, 3160834842u,  628085408u, 1382605366u,
  3423369109u, 3138078467u,  570562233u, 1426400815u, 3317316542u, 2998733608u,
   733239954u, 1555261956u, 3268935591u, 3050360625u,  752459403u, 1541320221u,
  2607071920u, 3965973030u, 1969922972u,   40735498u, 2617837225u, 3943577151u,
  1913087877u,   83908371u, 2512341634u, 3803740692u, 2075208622u,  213261112u,
  2463272603u, 3855990285u, 2094854071u,  198958881u, 2262029012u, 4057260610u,
  1759359992u,  534414190u, 2176718541u, 4139329115u, 1873836001u,  414664567u,
  2282248934u, 4279200368u, 1711684554u,  285281116u, 2405801727u, 4167216745u,
  1634467795u,  376229701u, 2685067896u, 3608007406u, 1308918612u,  956543938u,
  2808555105u, 3495958263u, 1231636301u, 1047427035u, 2932959818u, 3654703836u,
  1088359270u,  936918000u, 2847714899u, 3736837829u, 1202900863u,  817233897u,
  3183342108u, 3401237130u, 1404277552u,  615818150u, 3134207493u, 3453421203u,
  1423857449u,  601450431u, 3009837614u, 3294710456u, 1567103746u,  711928724u,
  3020668471u, 3272380065u, 1510334235u,  755167117u
};
static unsigned long CRC(const unsigned char* data, size_t size) {
  unsigned long result = 0xffffffffu;
  for (; size > 0; size--) {
    result = crc32_table[(result ^ *(data++)) & 0xff] ^ (result >> 8);
  }
  return result ^ 0xffffffffu;
}
void ZopfliGzipCompress(const ZopfliOptions* options,
                        const unsigned char* in, size_t insize,
                        unsigned char** out, size_t* outsize) {
  unsigned long crcvalue = CRC(in, insize);
  unsigned char bp = 0;
  ZOPFLI_APPEND_DATA(31, out, outsize);
  ZOPFLI_APPEND_DATA(139, out, outsize);
  ZOPFLI_APPEND_DATA(8, out, outsize);
  ZOPFLI_APPEND_DATA(0, out, outsize);
  ZOPFLI_APPEND_DATA(0, out, outsize);
  ZOPFLI_APPEND_DATA(0, out, outsize);
  ZOPFLI_APPEND_DATA(0, out, outsize);
  ZOPFLI_APPEND_DATA(0, out, outsize);
  ZOPFLI_APPEND_DATA(2, out, outsize);
  ZOPFLI_APPEND_DATA(3, out, outsize);
  ZopfliDeflate(options, 2 , 1,
                in, insize, &bp, out, outsize);
  ZOPFLI_APPEND_DATA(crcvalue % 256, out, outsize);
  ZOPFLI_APPEND_DATA((crcvalue >> 8) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((crcvalue >> 16) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((crcvalue >> 24) % 256, out, outsize);
  ZOPFLI_APPEND_DATA(insize % 256, out, outsize);
  ZOPFLI_APPEND_DATA((insize >> 8) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((insize >> 16) % 256, out, outsize);
  ZOPFLI_APPEND_DATA((insize >> 24) % 256, out, outsize);
  if (options->verbose) {
    fprintf(stderr,
            "Original Size: %d, Gzip: %d, Compression: %f%% Removed\n",
            (int)insize, (int)*outsize,
            100.0 * (double)(insize - *outsize) / (double)insize);
  }
}
```

## File: src/zopfli/hash.c
```cpp
#include "hash.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#define HASH_SHIFT 5
#define HASH_MASK 32767
void ZopfliAllocHash(size_t window_size, ZopfliHash* h) {
  h->head = (int*)malloc(sizeof(*h->head) * 65536);
  h->prev = (unsigned short*)malloc(sizeof(*h->prev) * window_size);
  h->hashval = (int*)malloc(sizeof(*h->hashval) * window_size);
#ifdef ZOPFLI_HASH_SAME
  h->same = (unsigned short*)malloc(sizeof(*h->same) * window_size);
#endif
#ifdef ZOPFLI_HASH_SAME_HASH
  h->head2 = (int*)malloc(sizeof(*h->head2) * 65536);
  h->prev2 = (unsigned short*)malloc(sizeof(*h->prev2) * window_size);
  h->hashval2 = (int*)malloc(sizeof(*h->hashval2) * window_size);
#endif
}
void ZopfliResetHash(size_t window_size, ZopfliHash* h) {
  size_t i;
  h->val = 0;
  for (i = 0; i < 65536; i++) {
    h->head[i] = -1;
  }
  for (i = 0; i < window_size; i++) {
    h->prev[i] = i;
    h->hashval[i] = -1;
  }
#ifdef ZOPFLI_HASH_SAME
  for (i = 0; i < window_size; i++) {
    h->same[i] = 0;
  }
#endif
#ifdef ZOPFLI_HASH_SAME_HASH
  h->val2 = 0;
  for (i = 0; i < 65536; i++) {
    h->head2[i] = -1;
  }
  for (i = 0; i < window_size; i++) {
    h->prev2[i] = i;
    h->hashval2[i] = -1;
  }
#endif
}
void ZopfliCleanHash(ZopfliHash* h) {
  free(h->head);
  free(h->prev);
  free(h->hashval);
#ifdef ZOPFLI_HASH_SAME_HASH
  free(h->head2);
  free(h->prev2);
  free(h->hashval2);
#endif
#ifdef ZOPFLI_HASH_SAME
  free(h->same);
#endif
}
static void UpdateHashValue(ZopfliHash* h, unsigned char c) {
  h->val = (((h->val) << HASH_SHIFT) ^ (c)) & HASH_MASK;
}
void ZopfliUpdateHash(const unsigned char* array, size_t pos, size_t end,
                ZopfliHash* h) {
  unsigned short hpos = pos & ZOPFLI_WINDOW_MASK;
#ifdef ZOPFLI_HASH_SAME
  size_t amount = 0;
#endif
  UpdateHashValue(h, pos + ZOPFLI_MIN_MATCH <= end ?
      array[pos + ZOPFLI_MIN_MATCH - 1] : 0);
  h->hashval[hpos] = h->val;
  if (h->head[h->val] != -1 && h->hashval[h->head[h->val]] == h->val) {
    h->prev[hpos] = h->head[h->val];
  }
  else h->prev[hpos] = hpos;
  h->head[h->val] = hpos;
#ifdef ZOPFLI_HASH_SAME
  if (h->same[(pos - 1) & ZOPFLI_WINDOW_MASK] > 1) {
    amount = h->same[(pos - 1) & ZOPFLI_WINDOW_MASK] - 1;
  }
  while (pos + amount + 1 < end &&
      array[pos] == array[pos + amount + 1] && amount < (unsigned short)(-1)) {
    amount++;
  }
  h->same[hpos] = amount;
#endif
#ifdef ZOPFLI_HASH_SAME_HASH
  h->val2 = ((h->same[hpos] - ZOPFLI_MIN_MATCH) & 255) ^ h->val;
  h->hashval2[hpos] = h->val2;
  if (h->head2[h->val2] != -1 && h->hashval2[h->head2[h->val2]] == h->val2) {
    h->prev2[hpos] = h->head2[h->val2];
  }
  else h->prev2[hpos] = hpos;
  h->head2[h->val2] = hpos;
#endif
}
void ZopfliWarmupHash(const unsigned char* array, size_t pos, size_t end,
                ZopfliHash* h) {
  UpdateHashValue(h, array[pos + 0]);
  if (pos + 1 < end) UpdateHashValue(h, array[pos + 1]);
}
```

## File: src/zopfli/hash.h
```
#ifndef ZOPFLI_HASH_H_
#define ZOPFLI_HASH_H_
#include "util.h"
typedef struct ZopfliHash {
  int* head;
  unsigned short* prev;
  int* hashval;
  int val;
#ifdef ZOPFLI_HASH_SAME_HASH
  int* head2;
  unsigned short* prev2;
  int* hashval2;
  int val2;
#endif
#ifdef ZOPFLI_HASH_SAME
  unsigned short* same;
#endif
} ZopfliHash;
void ZopfliAllocHash(size_t window_size, ZopfliHash* h);
void ZopfliResetHash(size_t window_size, ZopfliHash* h);
void ZopfliCleanHash(ZopfliHash* h);
void ZopfliUpdateHash(const unsigned char* array, size_t pos, size_t end,
                      ZopfliHash* h);
void ZopfliWarmupHash(const unsigned char* array, size_t pos, size_t end,
                      ZopfliHash* h);
#endif
```

## File: src/zopfli/lz77.h
```
#ifndef ZOPFLI_LZ77_H_
#define ZOPFLI_LZ77_H_
#include <stdlib.h>
#include "cache.h"
#include "hash.h"
#include "zopfli.h"
typedef struct ZopfliLZ77Store {
  unsigned short* litlens;
  unsigned short* dists;
  size_t size;
  const unsigned char* data;
  size_t* pos;
  unsigned short* ll_symbol;
  unsigned short* d_symbol;
  size_t* ll_counts;
  size_t* d_counts;
} ZopfliLZ77Store;
void ZopfliInitLZ77Store(const unsigned char* data, ZopfliLZ77Store* store);
void ZopfliCleanLZ77Store(ZopfliLZ77Store* store);
void ZopfliCopyLZ77Store(const ZopfliLZ77Store* source, ZopfliLZ77Store* dest);
void ZopfliStoreLitLenDist(unsigned short length, unsigned short dist,
                           size_t pos, ZopfliLZ77Store* store);
void ZopfliAppendLZ77Store(const ZopfliLZ77Store* store,
                           ZopfliLZ77Store* target);
size_t ZopfliLZ77GetByteRange(const ZopfliLZ77Store* lz77,
                              size_t lstart, size_t lend);
void ZopfliLZ77GetHistogram(const ZopfliLZ77Store* lz77,
                            size_t lstart, size_t lend,
                            size_t* ll_counts, size_t* d_counts);
typedef struct ZopfliBlockState {
  const ZopfliOptions* options;
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
  ZopfliLongestMatchCache* lmc;
#endif
  size_t blockstart;
  size_t blockend;
} ZopfliBlockState;
void ZopfliInitBlockState(const ZopfliOptions* options,
                          size_t blockstart, size_t blockend, int add_lmc,
                          ZopfliBlockState* s);
void ZopfliCleanBlockState(ZopfliBlockState* s);
void ZopfliFindLongestMatch(
    ZopfliBlockState *s, const ZopfliHash* h, const unsigned char* array,
    size_t pos, size_t size, size_t limit,
    unsigned short* sublen, unsigned short* distance, unsigned short* length);
void ZopfliVerifyLenDist(const unsigned char* data, size_t datasize, size_t pos,
                         unsigned short dist, unsigned short length);
void ZopfliLZ77Greedy(ZopfliBlockState* s, const unsigned char* in,
                      size_t instart, size_t inend,
                      ZopfliLZ77Store* store, ZopfliHash* h);
#endif
```

## File: src/zopflipng/zopflipng_lib.h
```
#ifndef ZOPFLIPNG_LIB_H_
#define ZOPFLIPNG_LIB_H_
#ifdef __cplusplus
#include <string>
#include <vector>
extern "C" {
#endif
#include <stdlib.h>
enum ZopfliPNGFilterStrategy {
  kStrategyZero = 0,
  kStrategyOne = 1,
  kStrategyTwo = 2,
  kStrategyThree = 3,
  kStrategyFour = 4,
  kStrategyMinSum,
  kStrategyEntropy,
  kStrategyPredefined,
  kStrategyBruteForce,
  kNumFilterStrategies
};
typedef struct CZopfliPNGOptions {
  int lossy_transparent;
  int lossy_8bit;
  enum ZopfliPNGFilterStrategy* filter_strategies;
  int num_filter_strategies;
  int auto_filter_strategy;
  char** keepchunks;
  int num_keepchunks;
  int use_zopfli;
  int num_iterations;
  int num_iterations_large;
  int block_split_strategy;
} CZopfliPNGOptions;
void CZopfliPNGSetDefaults(CZopfliPNGOptions *png_options);
int CZopfliPNGOptimize(const unsigned char* origpng,
    const size_t origpng_size,
    const CZopfliPNGOptions* png_options,
    int verbose,
    unsigned char** resultpng,
    size_t* resultpng_size);
#ifdef __cplusplus
}
#endif
#ifdef __cplusplus
struct ZopfliPNGOptions {
  ZopfliPNGOptions();
  bool verbose;
  bool lossy_transparent;
  bool lossy_8bit;
  std::vector<ZopfliPNGFilterStrategy> filter_strategies;
  bool auto_filter_strategy;
  bool keep_colortype;
  std::vector<std::string> keepchunks;
  bool use_zopfli;
  int num_iterations;
  int num_iterations_large;
  int block_split_strategy;
};
int ZopfliPNGOptimize(const std::vector<unsigned char>& origpng,
    const ZopfliPNGOptions& png_options,
    bool verbose,
    std::vector<unsigned char>* resultpng);
#endif
#endif
```

## File: src/zopfli/cache.c
```cpp
#include "cache.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
void ZopfliInitCache(size_t blocksize, ZopfliLongestMatchCache* lmc) {
  size_t i;
  lmc->length = (unsigned short*)malloc(sizeof(unsigned short) * blocksize);
  lmc->dist = (unsigned short*)malloc(sizeof(unsigned short) * blocksize);
  lmc->sublen = (unsigned char*)malloc(ZOPFLI_CACHE_LENGTH * 3 * blocksize);
  if(lmc->sublen == NULL) {
    fprintf(stderr,
        "Error: Out of memory. Tried allocating %lu bytes of memory.\n",
        (unsigned long)ZOPFLI_CACHE_LENGTH * 3 * blocksize);
    exit (EXIT_FAILURE);
  }
  for (i = 0; i < blocksize; i++) lmc->length[i] = 1;
  for (i = 0; i < blocksize; i++) lmc->dist[i] = 0;
  for (i = 0; i < ZOPFLI_CACHE_LENGTH * blocksize * 3; i++) lmc->sublen[i] = 0;
}
void ZopfliCleanCache(ZopfliLongestMatchCache* lmc) {
  free(lmc->length);
  free(lmc->dist);
  free(lmc->sublen);
}
void ZopfliSublenToCache(const unsigned short* sublen,
                         size_t pos, size_t length,
                         ZopfliLongestMatchCache* lmc) {
  size_t i;
  size_t j = 0;
  unsigned bestlength = 0;
  unsigned char* cache;
#if ZOPFLI_CACHE_LENGTH == 0
  return;
#endif
  cache = &lmc->sublen[ZOPFLI_CACHE_LENGTH * pos * 3];
  if (length < 3) return;
  for (i = 3; i <= length; i++) {
    if (i == length || sublen[i] != sublen[i + 1]) {
      cache[j * 3] = i - 3;
      cache[j * 3 + 1] = sublen[i] % 256;
      cache[j * 3 + 2] = (sublen[i] >> 8) % 256;
      bestlength = i;
      j++;
      if (j >= ZOPFLI_CACHE_LENGTH) break;
    }
  }
  if (j < ZOPFLI_CACHE_LENGTH) {
    assert(bestlength == length);
    cache[(ZOPFLI_CACHE_LENGTH - 1) * 3] = bestlength - 3;
  } else {
    assert(bestlength <= length);
  }
  assert(bestlength == ZopfliMaxCachedSublen(lmc, pos, length));
}
void ZopfliCacheToSublen(const ZopfliLongestMatchCache* lmc,
                         size_t pos, size_t length,
                         unsigned short* sublen) {
  size_t i, j;
  unsigned maxlength = ZopfliMaxCachedSublen(lmc, pos, length);
  unsigned prevlength = 0;
  unsigned char* cache;
#if ZOPFLI_CACHE_LENGTH == 0
  return;
#endif
  if (length < 3) return;
  cache = &lmc->sublen[ZOPFLI_CACHE_LENGTH * pos * 3];
  for (j = 0; j < ZOPFLI_CACHE_LENGTH; j++) {
    unsigned length = cache[j * 3] + 3;
    unsigned dist = cache[j * 3 + 1] + 256 * cache[j * 3 + 2];
    for (i = prevlength; i <= length; i++) {
      sublen[i] = dist;
    }
    if (length == maxlength) break;
    prevlength = length + 1;
  }
}
unsigned ZopfliMaxCachedSublen(const ZopfliLongestMatchCache* lmc,
                               size_t pos, size_t length) {
  unsigned char* cache;
#if ZOPFLI_CACHE_LENGTH == 0
  return 0;
#endif
  cache = &lmc->sublen[ZOPFLI_CACHE_LENGTH * pos * 3];
  (void)length;
  if (cache[1] == 0 && cache[2] == 0) return 0;
  return cache[(ZOPFLI_CACHE_LENGTH - 1) * 3] + 3;
}
#endif
```

## File: src/zopfli/katajainen.c
```cpp
#include "katajainen.h"
#include <assert.h>
#include <stdlib.h>
#include <limits.h>
typedef struct Node Node;
struct Node {
  size_t weight;
  Node* tail;
  int count;
};
typedef struct NodePool {
  Node* next;
} NodePool;
static void InitNode(size_t weight, int count, Node* tail, Node* node) {
  node->weight = weight;
  node->count = count;
  node->tail = tail;
}
static void BoundaryPM(Node* (*lists)[2], Node* leaves, int numsymbols,
                       NodePool* pool, int index) {
  Node* newchain;
  Node* oldchain;
  int lastcount = lists[index][1]->count;
  if (index == 0 && lastcount >= numsymbols) return;
  newchain = pool->next++;
  oldchain = lists[index][1];
  lists[index][0] = oldchain;
  lists[index][1] = newchain;
  if (index == 0) {
    InitNode(leaves[lastcount].weight, lastcount + 1, 0, newchain);
  } else {
    size_t sum = lists[index - 1][0]->weight + lists[index - 1][1]->weight;
    if (lastcount < numsymbols && sum > leaves[lastcount].weight) {
      InitNode(leaves[lastcount].weight, lastcount + 1, oldchain->tail,
          newchain);
    } else {
      InitNode(sum, lastcount, lists[index - 1][1], newchain);
      BoundaryPM(lists, leaves, numsymbols, pool, index - 1);
      BoundaryPM(lists, leaves, numsymbols, pool, index - 1);
    }
  }
}
static void BoundaryPMFinal(Node* (*lists)[2],
    Node* leaves, int numsymbols, NodePool* pool, int index) {
  int lastcount = lists[index][1]->count;
  size_t sum = lists[index - 1][0]->weight + lists[index - 1][1]->weight;
  if (lastcount < numsymbols && sum > leaves[lastcount].weight) {
    Node* newchain = pool->next;
    Node* oldchain = lists[index][1]->tail;
    lists[index][1] = newchain;
    newchain->count = lastcount + 1;
    newchain->tail = oldchain;
  } else {
    lists[index][1]->tail = lists[index - 1][1];
  }
}
static void InitLists(
    NodePool* pool, const Node* leaves, int maxbits, Node* (*lists)[2]) {
  int i;
  Node* node0 = pool->next++;
  Node* node1 = pool->next++;
  InitNode(leaves[0].weight, 1, 0, node0);
  InitNode(leaves[1].weight, 2, 0, node1);
  for (i = 0; i < maxbits; i++) {
    lists[i][0] = node0;
    lists[i][1] = node1;
  }
}
static void ExtractBitLengths(Node* chain, Node* leaves, unsigned* bitlengths) {
  int counts[16] = {0};
  unsigned end = 16;
  unsigned ptr = 15;
  unsigned value = 1;
  Node* node;
  int val;
  for (node = chain; node; node = node->tail) {
    counts[--end] = node->count;
  }
  val = counts[15];
  while (ptr >= end) {
    for (; val > counts[ptr - 1]; val--) {
      bitlengths[leaves[val - 1].count] = value;
    }
    ptr--;
    value++;
  }
}
static int LeafComparator(const void* a, const void* b) {
  return ((const Node*)a)->weight - ((const Node*)b)->weight;
}
int ZopfliLengthLimitedCodeLengths(
    const size_t* frequencies, int n, int maxbits, unsigned* bitlengths) {
  NodePool pool;
  int i;
  int numsymbols = 0;
  int numBoundaryPMRuns;
  Node* nodes;
  Node* (*lists)[2];
  Node* leaves = (Node*)malloc(n * sizeof(*leaves));
  for (i = 0; i < n; i++) {
    bitlengths[i] = 0;
  }
  for (i = 0; i < n; i++) {
    if (frequencies[i]) {
      leaves[numsymbols].weight = frequencies[i];
      leaves[numsymbols].count = i;
      numsymbols++;
    }
  }
  if ((1 << maxbits) < numsymbols) {
    free(leaves);
    return 1;
  }
  if (numsymbols == 0) {
    free(leaves);
    return 0;
  }
  if (numsymbols == 1) {
    bitlengths[leaves[0].count] = 1;
    free(leaves);
    return 0;
  }
  if (numsymbols == 2) {
    bitlengths[leaves[0].count]++;
    bitlengths[leaves[1].count]++;
    free(leaves);
    return 0;
  }
  for (i = 0; i < numsymbols; i++) {
    if (leaves[i].weight >=
        ((size_t)1 << (sizeof(leaves[0].weight) * CHAR_BIT - 9))) {
      free(leaves);
      return 1;
    }
    leaves[i].weight = (leaves[i].weight << 9) | leaves[i].count;
  }
  qsort(leaves, numsymbols, sizeof(Node), LeafComparator);
  for (i = 0; i < numsymbols; i++) {
    leaves[i].weight >>= 9;
  }
  if (numsymbols - 1 < maxbits) {
    maxbits = numsymbols - 1;
  }
  nodes = (Node*)malloc(maxbits * 2 * numsymbols * sizeof(Node));
  pool.next = nodes;
  lists = (Node* (*)[2])malloc(maxbits * sizeof(*lists));
  InitLists(&pool, leaves, maxbits, lists);
  numBoundaryPMRuns = 2 * numsymbols - 4;
  for (i = 0; i < numBoundaryPMRuns - 1; i++) {
    BoundaryPM(lists, leaves, numsymbols, &pool, maxbits - 1);
  }
  BoundaryPMFinal(lists, leaves, numsymbols, &pool, maxbits - 1);
  ExtractBitLengths(lists[maxbits - 1][1], leaves, bitlengths);
  free(lists);
  free(leaves);
  free(nodes);
  return 0;
}
```

## File: src/zopfli/lz77.c
```cpp
#include "lz77.h"
#include "symbols.h"
#include "util.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
void ZopfliInitLZ77Store(const unsigned char* data, ZopfliLZ77Store* store) {
  store->size = 0;
  store->litlens = 0;
  store->dists = 0;
  store->pos = 0;
  store->data = data;
  store->ll_symbol = 0;
  store->d_symbol = 0;
  store->ll_counts = 0;
  store->d_counts = 0;
}
void ZopfliCleanLZ77Store(ZopfliLZ77Store* store) {
  free(store->litlens);
  free(store->dists);
  free(store->pos);
  free(store->ll_symbol);
  free(store->d_symbol);
  free(store->ll_counts);
  free(store->d_counts);
}
static size_t CeilDiv(size_t a, size_t b) {
  return (a + b - 1) / b;
}
void ZopfliCopyLZ77Store(
    const ZopfliLZ77Store* source, ZopfliLZ77Store* dest) {
  size_t i;
  size_t llsize = ZOPFLI_NUM_LL * CeilDiv(source->size, ZOPFLI_NUM_LL);
  size_t dsize = ZOPFLI_NUM_D * CeilDiv(source->size, ZOPFLI_NUM_D);
  ZopfliCleanLZ77Store(dest);
  ZopfliInitLZ77Store(source->data, dest);
  dest->litlens =
      (unsigned short*)malloc(sizeof(*dest->litlens) * source->size);
  dest->dists = (unsigned short*)malloc(sizeof(*dest->dists) * source->size);
  dest->pos = (size_t*)malloc(sizeof(*dest->pos) * source->size);
  dest->ll_symbol =
      (unsigned short*)malloc(sizeof(*dest->ll_symbol) * source->size);
  dest->d_symbol =
      (unsigned short*)malloc(sizeof(*dest->d_symbol) * source->size);
  dest->ll_counts = (size_t*)malloc(sizeof(*dest->ll_counts) * llsize);
  dest->d_counts = (size_t*)malloc(sizeof(*dest->d_counts) * dsize);
  if (!dest->litlens || !dest->dists) exit(-1);
  if (!dest->pos) exit(-1);
  if (!dest->ll_symbol || !dest->d_symbol) exit(-1);
  if (!dest->ll_counts || !dest->d_counts) exit(-1);
  dest->size = source->size;
  for (i = 0; i < source->size; i++) {
    dest->litlens[i] = source->litlens[i];
    dest->dists[i] = source->dists[i];
    dest->pos[i] = source->pos[i];
    dest->ll_symbol[i] = source->ll_symbol[i];
    dest->d_symbol[i] = source->d_symbol[i];
  }
  for (i = 0; i < llsize; i++) {
    dest->ll_counts[i] = source->ll_counts[i];
  }
  for (i = 0; i < dsize; i++) {
    dest->d_counts[i] = source->d_counts[i];
  }
}
void ZopfliStoreLitLenDist(unsigned short length, unsigned short dist,
                           size_t pos, ZopfliLZ77Store* store) {
  size_t i;
  size_t origsize = store->size;
  size_t llstart = ZOPFLI_NUM_LL * (origsize / ZOPFLI_NUM_LL);
  size_t dstart = ZOPFLI_NUM_D * (origsize / ZOPFLI_NUM_D);
  if (origsize % ZOPFLI_NUM_LL == 0) {
    size_t llsize = origsize;
    for (i = 0; i < ZOPFLI_NUM_LL; i++) {
      ZOPFLI_APPEND_DATA(
          origsize == 0 ? 0 : store->ll_counts[origsize - ZOPFLI_NUM_LL + i],
          &store->ll_counts, &llsize);
    }
  }
  if (origsize % ZOPFLI_NUM_D == 0) {
    size_t dsize = origsize;
    for (i = 0; i < ZOPFLI_NUM_D; i++) {
      ZOPFLI_APPEND_DATA(
          origsize == 0 ? 0 : store->d_counts[origsize - ZOPFLI_NUM_D + i],
          &store->d_counts, &dsize);
    }
  }
  ZOPFLI_APPEND_DATA(length, &store->litlens, &store->size);
  store->size = origsize;
  ZOPFLI_APPEND_DATA(dist, &store->dists, &store->size);
  store->size = origsize;
  ZOPFLI_APPEND_DATA(pos, &store->pos, &store->size);
  assert(length < 259);
  if (dist == 0) {
    store->size = origsize;
    ZOPFLI_APPEND_DATA(length, &store->ll_symbol, &store->size);
    store->size = origsize;
    ZOPFLI_APPEND_DATA(0, &store->d_symbol, &store->size);
    store->ll_counts[llstart + length]++;
  } else {
    store->size = origsize;
    ZOPFLI_APPEND_DATA(ZopfliGetLengthSymbol(length),
                       &store->ll_symbol, &store->size);
    store->size = origsize;
    ZOPFLI_APPEND_DATA(ZopfliGetDistSymbol(dist),
                       &store->d_symbol, &store->size);
    store->ll_counts[llstart + ZopfliGetLengthSymbol(length)]++;
    store->d_counts[dstart + ZopfliGetDistSymbol(dist)]++;
  }
}
void ZopfliAppendLZ77Store(const ZopfliLZ77Store* store,
                           ZopfliLZ77Store* target) {
  size_t i;
  for (i = 0; i < store->size; i++) {
    ZopfliStoreLitLenDist(store->litlens[i], store->dists[i],
                          store->pos[i], target);
  }
}
size_t ZopfliLZ77GetByteRange(const ZopfliLZ77Store* lz77,
                              size_t lstart, size_t lend) {
  size_t l = lend - 1;
  if (lstart == lend) return 0;
  return lz77->pos[l] + ((lz77->dists[l] == 0) ?
      1 : lz77->litlens[l]) - lz77->pos[lstart];
}
static void ZopfliLZ77GetHistogramAt(const ZopfliLZ77Store* lz77, size_t lpos,
                                     size_t* ll_counts, size_t* d_counts) {
  size_t llpos = ZOPFLI_NUM_LL * (lpos / ZOPFLI_NUM_LL);
  size_t dpos = ZOPFLI_NUM_D * (lpos / ZOPFLI_NUM_D);
  size_t i;
  for (i = 0; i < ZOPFLI_NUM_LL; i++) {
    ll_counts[i] = lz77->ll_counts[llpos + i];
  }
  for (i = lpos + 1; i < llpos + ZOPFLI_NUM_LL && i < lz77->size; i++) {
    ll_counts[lz77->ll_symbol[i]]--;
  }
  for (i = 0; i < ZOPFLI_NUM_D; i++) {
    d_counts[i] = lz77->d_counts[dpos + i];
  }
  for (i = lpos + 1; i < dpos + ZOPFLI_NUM_D && i < lz77->size; i++) {
    if (lz77->dists[i] != 0) d_counts[lz77->d_symbol[i]]--;
  }
}
void ZopfliLZ77GetHistogram(const ZopfliLZ77Store* lz77,
                           size_t lstart, size_t lend,
                           size_t* ll_counts, size_t* d_counts) {
  size_t i;
  if (lstart + ZOPFLI_NUM_LL * 3 > lend) {
    memset(ll_counts, 0, sizeof(*ll_counts) * ZOPFLI_NUM_LL);
    memset(d_counts, 0, sizeof(*d_counts) * ZOPFLI_NUM_D);
    for (i = lstart; i < lend; i++) {
      ll_counts[lz77->ll_symbol[i]]++;
      if (lz77->dists[i] != 0) d_counts[lz77->d_symbol[i]]++;
    }
  } else {
    ZopfliLZ77GetHistogramAt(lz77, lend - 1, ll_counts, d_counts);
    if (lstart > 0) {
      size_t ll_counts2[ZOPFLI_NUM_LL];
      size_t d_counts2[ZOPFLI_NUM_D];
      ZopfliLZ77GetHistogramAt(lz77, lstart - 1, ll_counts2, d_counts2);
      for (i = 0; i < ZOPFLI_NUM_LL; i++) {
        ll_counts[i] -= ll_counts2[i];
      }
      for (i = 0; i < ZOPFLI_NUM_D; i++) {
        d_counts[i] -= d_counts2[i];
      }
    }
  }
}
void ZopfliInitBlockState(const ZopfliOptions* options,
                          size_t blockstart, size_t blockend, int add_lmc,
                          ZopfliBlockState* s) {
  s->options = options;
  s->blockstart = blockstart;
  s->blockend = blockend;
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
  if (add_lmc) {
    s->lmc = (ZopfliLongestMatchCache*)malloc(sizeof(ZopfliLongestMatchCache));
    ZopfliInitCache(blockend - blockstart, s->lmc);
  } else {
    s->lmc = 0;
  }
#endif
}
void ZopfliCleanBlockState(ZopfliBlockState* s) {
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
  if (s->lmc) {
    ZopfliCleanCache(s->lmc);
    free(s->lmc);
  }
#endif
}
static int GetLengthScore(int length, int distance) {
  return distance > 1024 ? length - 1 : length;
}
void ZopfliVerifyLenDist(const unsigned char* data, size_t datasize, size_t pos,
                         unsigned short dist, unsigned short length) {
  size_t i;
  assert(pos + length <= datasize);
  for (i = 0; i < length; i++) {
    if (data[pos - dist + i] != data[pos + i]) {
      assert(data[pos - dist + i] == data[pos + i]);
      break;
    }
  }
}
static const unsigned char* GetMatch(const unsigned char* scan,
                                     const unsigned char* match,
                                     const unsigned char* end,
                                     const unsigned char* safe_end) {
  if (sizeof(size_t) == 8) {
    while (scan < safe_end && *((size_t*)scan) == *((size_t*)match)) {
      scan += 8;
      match += 8;
    }
  } else if (sizeof(unsigned int) == 4) {
    while (scan < safe_end
        && *((unsigned int*)scan) == *((unsigned int*)match)) {
      scan += 4;
      match += 4;
    }
  } else {
    while (scan < safe_end && *scan == *match && *++scan == *++match
          && *++scan == *++match && *++scan == *++match
          && *++scan == *++match && *++scan == *++match
          && *++scan == *++match && *++scan == *++match) {
      scan++; match++;
    }
  }
  while (scan != end && *scan == *match) {
    scan++; match++;
  }
  return scan;
}
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
static int TryGetFromLongestMatchCache(ZopfliBlockState* s,
    size_t pos, size_t* limit,
    unsigned short* sublen, unsigned short* distance, unsigned short* length) {
  size_t lmcpos = pos - s->blockstart;
  unsigned char cache_available = s->lmc && (s->lmc->length[lmcpos] == 0 ||
      s->lmc->dist[lmcpos] != 0);
  unsigned char limit_ok_for_cache = cache_available &&
      (*limit == ZOPFLI_MAX_MATCH || s->lmc->length[lmcpos] <= *limit ||
      (sublen && ZopfliMaxCachedSublen(s->lmc,
          lmcpos, s->lmc->length[lmcpos]) >= *limit));
  if (s->lmc && limit_ok_for_cache && cache_available) {
    if (!sublen || s->lmc->length[lmcpos]
        <= ZopfliMaxCachedSublen(s->lmc, lmcpos, s->lmc->length[lmcpos])) {
      *length = s->lmc->length[lmcpos];
      if (*length > *limit) *length = *limit;
      if (sublen) {
        ZopfliCacheToSublen(s->lmc, lmcpos, *length, sublen);
        *distance = sublen[*length];
        if (*limit == ZOPFLI_MAX_MATCH && *length >= ZOPFLI_MIN_MATCH) {
          assert(sublen[*length] == s->lmc->dist[lmcpos]);
        }
      } else {
        *distance = s->lmc->dist[lmcpos];
      }
      return 1;
    }
    *limit = s->lmc->length[lmcpos];
  }
  return 0;
}
static void StoreInLongestMatchCache(ZopfliBlockState* s,
    size_t pos, size_t limit,
    const unsigned short* sublen,
    unsigned short distance, unsigned short length) {
  size_t lmcpos = pos - s->blockstart;
  unsigned char cache_available = s->lmc && (s->lmc->length[lmcpos] == 0 ||
      s->lmc->dist[lmcpos] != 0);
  if (s->lmc && limit == ZOPFLI_MAX_MATCH && sublen && !cache_available) {
    assert(s->lmc->length[lmcpos] == 1 && s->lmc->dist[lmcpos] == 0);
    s->lmc->dist[lmcpos] = length < ZOPFLI_MIN_MATCH ? 0 : distance;
    s->lmc->length[lmcpos] = length < ZOPFLI_MIN_MATCH ? 0 : length;
    assert(!(s->lmc->length[lmcpos] == 1 && s->lmc->dist[lmcpos] == 0));
    ZopfliSublenToCache(sublen, lmcpos, length, s->lmc);
  }
}
#endif
void ZopfliFindLongestMatch(ZopfliBlockState* s, const ZopfliHash* h,
    const unsigned char* array,
    size_t pos, size_t size, size_t limit,
    unsigned short* sublen, unsigned short* distance, unsigned short* length) {
  unsigned short hpos = pos & ZOPFLI_WINDOW_MASK, p, pp;
  unsigned short bestdist = 0;
  unsigned short bestlength = 1;
  const unsigned char* scan;
  const unsigned char* match;
  const unsigned char* arrayend;
  const unsigned char* arrayend_safe;
#if ZOPFLI_MAX_CHAIN_HITS < ZOPFLI_WINDOW_SIZE
  int chain_counter = ZOPFLI_MAX_CHAIN_HITS;
#endif
  unsigned dist = 0;
  int* hhead = h->head;
  unsigned short* hprev = h->prev;
  int* hhashval = h->hashval;
  int hval = h->val;
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
  if (TryGetFromLongestMatchCache(s, pos, &limit, sublen, distance, length)) {
    assert(pos + *length <= size);
    return;
  }
#endif
  assert(limit <= ZOPFLI_MAX_MATCH);
  assert(limit >= ZOPFLI_MIN_MATCH);
  assert(pos < size);
  if (size - pos < ZOPFLI_MIN_MATCH) {
    *length = 0;
    *distance = 0;
    return;
  }
  if (pos + limit > size) {
    limit = size - pos;
  }
  arrayend = &array[pos] + limit;
  arrayend_safe = arrayend - 8;
  assert(hval < 65536);
  pp = hhead[hval];
  p = hprev[pp];
  assert(pp == hpos);
  dist = p < pp ? pp - p : ((ZOPFLI_WINDOW_SIZE - p) + pp);
  while (dist < ZOPFLI_WINDOW_SIZE) {
    unsigned short currentlength = 0;
    assert(p < ZOPFLI_WINDOW_SIZE);
    assert(p == hprev[pp]);
    assert(hhashval[p] == hval);
    if (dist > 0) {
      assert(pos < size);
      assert(dist <= pos);
      scan = &array[pos];
      match = &array[pos - dist];
      if (pos + bestlength >= size
          || *(scan + bestlength) == *(match + bestlength)) {
#ifdef ZOPFLI_HASH_SAME
        unsigned short same0 = h->same[pos & ZOPFLI_WINDOW_MASK];
        if (same0 > 2 && *scan == *match) {
          unsigned short same1 = h->same[(pos - dist) & ZOPFLI_WINDOW_MASK];
          unsigned short same = same0 < same1 ? same0 : same1;
          if (same > limit) same = limit;
          scan += same;
          match += same;
        }
#endif
        scan = GetMatch(scan, match, arrayend, arrayend_safe);
        currentlength = scan - &array[pos];
      }
      if (currentlength > bestlength) {
        if (sublen) {
          unsigned short j;
          for (j = bestlength + 1; j <= currentlength; j++) {
            sublen[j] = dist;
          }
        }
        bestdist = dist;
        bestlength = currentlength;
        if (currentlength >= limit) break;
      }
    }
#ifdef ZOPFLI_HASH_SAME_HASH
    if (hhead != h->head2 && bestlength >= h->same[hpos] &&
        h->val2 == h->hashval2[p]) {
      hhead = h->head2;
      hprev = h->prev2;
      hhashval = h->hashval2;
      hval = h->val2;
    }
#endif
    pp = p;
    p = hprev[p];
    if (p == pp) break;
    dist += p < pp ? pp - p : ((ZOPFLI_WINDOW_SIZE - p) + pp);
#if ZOPFLI_MAX_CHAIN_HITS < ZOPFLI_WINDOW_SIZE
    chain_counter--;
    if (chain_counter <= 0) break;
#endif
  }
#ifdef ZOPFLI_LONGEST_MATCH_CACHE
  StoreInLongestMatchCache(s, pos, limit, sublen, bestdist, bestlength);
#endif
  assert(bestlength <= limit);
  *distance = bestdist;
  *length = bestlength;
  assert(pos + *length <= size);
}
void ZopfliLZ77Greedy(ZopfliBlockState* s, const unsigned char* in,
                      size_t instart, size_t inend,
                      ZopfliLZ77Store* store, ZopfliHash* h) {
  size_t i = 0, j;
  unsigned short leng;
  unsigned short dist;
  int lengthscore;
  size_t windowstart = instart > ZOPFLI_WINDOW_SIZE
      ? instart - ZOPFLI_WINDOW_SIZE : 0;
  unsigned short dummysublen[259];
#ifdef ZOPFLI_LAZY_MATCHING
  unsigned prev_length = 0;
  unsigned prev_match = 0;
  int prevlengthscore;
  int match_available = 0;
#endif
  if (instart == inend) return;
  ZopfliResetHash(ZOPFLI_WINDOW_SIZE, h);
  ZopfliWarmupHash(in, windowstart, inend, h);
  for (i = windowstart; i < instart; i++) {
    ZopfliUpdateHash(in, i, inend, h);
  }
  for (i = instart; i < inend; i++) {
    ZopfliUpdateHash(in, i, inend, h);
    ZopfliFindLongestMatch(s, h, in, i, inend, ZOPFLI_MAX_MATCH, dummysublen,
                           &dist, &leng);
    lengthscore = GetLengthScore(leng, dist);
#ifdef ZOPFLI_LAZY_MATCHING
    prevlengthscore = GetLengthScore(prev_length, prev_match);
    if (match_available) {
      match_available = 0;
      if (lengthscore > prevlengthscore + 1) {
        ZopfliStoreLitLenDist(in[i - 1], 0, i - 1, store);
        if (lengthscore >= ZOPFLI_MIN_MATCH && leng < ZOPFLI_MAX_MATCH) {
          match_available = 1;
          prev_length = leng;
          prev_match = dist;
          continue;
        }
      } else {
        leng = prev_length;
        dist = prev_match;
        lengthscore = prevlengthscore;
        ZopfliVerifyLenDist(in, inend, i - 1, dist, leng);
        ZopfliStoreLitLenDist(leng, dist, i - 1, store);
        for (j = 2; j < leng; j++) {
          assert(i < inend);
          i++;
          ZopfliUpdateHash(in, i, inend, h);
        }
        continue;
      }
    }
    else if (lengthscore >= ZOPFLI_MIN_MATCH && leng < ZOPFLI_MAX_MATCH) {
      match_available = 1;
      prev_length = leng;
      prev_match = dist;
      continue;
    }
#endif
    if (lengthscore >= ZOPFLI_MIN_MATCH) {
      ZopfliVerifyLenDist(in, inend, i, dist, leng);
      ZopfliStoreLitLenDist(leng, dist, i, store);
    } else {
      leng = 1;
      ZopfliStoreLitLenDist(in[i], 0, i, store);
    }
    for (j = 1; j < leng; j++) {
      assert(i < inend);
      i++;
      ZopfliUpdateHash(in, i, inend, h);
    }
  }
}
```

## File: src/zopfli/util.c
```cpp
#include "util.h"
#include "zopfli.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
void ZopfliInitOptions(ZopfliOptions* options) {
  options->verbose = 0;
  options->verbose_more = 0;
  options->numiterations = 15;
  options->blocksplitting = 1;
  options->blocksplittinglast = 0;
  options->blocksplittingmax = 15;
}
```

## File: src/zopfli/util.h
```
#ifndef ZOPFLI_UTIL_H_
#define ZOPFLI_UTIL_H_
#include <string.h>
#include <stdlib.h>
#define ZOPFLI_MAX_MATCH 258
#define ZOPFLI_MIN_MATCH 3
#define ZOPFLI_NUM_LL 288
#define ZOPFLI_NUM_D 32
#define ZOPFLI_WINDOW_SIZE 32768
#define ZOPFLI_WINDOW_MASK (ZOPFLI_WINDOW_SIZE - 1)
#define ZOPFLI_MASTER_BLOCK_SIZE 1000000
#define ZOPFLI_LARGE_FLOAT 1e30
#define ZOPFLI_CACHE_LENGTH 8
#define ZOPFLI_MAX_CHAIN_HITS 8192
#define ZOPFLI_LONGEST_MATCH_CACHE
#define ZOPFLI_HASH_SAME
#define ZOPFLI_HASH_SAME_HASH
#define ZOPFLI_SHORTCUT_LONG_REPETITIONS
#define ZOPFLI_LAZY_MATCHING
#ifdef __cplusplus
#define ZOPFLI_APPEND_DATA( value,  data,  size) {\
  if (!((*size) & ((*size) - 1))) {\
    \
    void** data_void = reinterpret_cast<void**>(data);\
    *data_void = (*size) == 0 ? malloc(sizeof(**data))\
                              : realloc((*data), (*size) * 2 * sizeof(**data));\
  }\
  (*data)[(*size)] = (value);\
  (*size)++;\
}
#else
#define ZOPFLI_APPEND_DATA( value,  data,  size) {\
  if (!((*size) & ((*size) - 1))) {\
    \
    (*data) = (*size) == 0 ? malloc(sizeof(**data))\
                           : realloc((*data), (*size) * 2 * sizeof(**data));\
  }\
  (*data)[(*size)] = (value);\
  (*size)++;\
}
#endif
#endif
```

## File: src/zopflipng/lodepng/lodepng_util.h
```
#ifndef LODEPNG_UTIL_H
#define LODEPNG_UTIL_H
#include <string>
#include <vector>
#include "lodepng.h"
namespace lodepng {
LodePNGInfo getPNGHeaderInfo(const std::vector<unsigned char>& png);
unsigned getChunkInfo(std::vector<std::string>& names, std::vector<size_t>& sizes,
                      const std::vector<unsigned char>& png);
unsigned getChunks(std::vector<std::string> names[3],
                   std::vector<std::vector<unsigned char> > chunks[3],
                   const std::vector<unsigned char>& png);
unsigned insertChunks(std::vector<unsigned char>& png,
                      const std::vector<std::vector<unsigned char> > chunks[3]);
unsigned getFilterTypes(std::vector<unsigned char>& filterTypes, const std::vector<unsigned char>& png);
unsigned getFilterTypesInterlaced(std::vector<std::vector<unsigned char> >& filterTypes,
                                  const std::vector<unsigned char>& png);
int getPaletteValue(const unsigned char* data, size_t i, int bits);
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
unsigned convertToSrgb(unsigned char* out, const unsigned char* in,
                       unsigned w, unsigned h,
                       const LodePNGState* state_in);
unsigned convertFromSrgb(unsigned char* out, const unsigned char* in,
                         unsigned w, unsigned h,
                         const LodePNGState* state_out);
unsigned convertRGBModel(unsigned char* out, const unsigned char* in,
                         unsigned w, unsigned h,
                         const LodePNGState* state_out,
                         const LodePNGState* state_in,
                         unsigned rendering_intent);
unsigned convertToXYZ(float* out, float whitepoint[3],
                      const unsigned char* in, unsigned w, unsigned h,
                      const LodePNGState* state);
unsigned convertToXYZFloat(float* out, float whitepoint[3], const float* in,
                           unsigned w, unsigned h, const LodePNGState* state);
unsigned convertFromXYZ(unsigned char* out, const float* in, unsigned w, unsigned h,
                        const LodePNGState* state,
                        const float whitepoint[3], unsigned rendering_intent);
unsigned convertFromXYZFloat(float* out, const float* in, unsigned w, unsigned h,
                             const LodePNGState* state,
                             const float whitepoint[3], unsigned rendering_intent);
#endif
struct ZlibBlockInfo {
  int btype;
  size_t compressedbits;
  size_t uncompressedbytes;
  size_t treebits;
  int hlit;
  int hdist;
  int hclen;
  std::vector<int> clcl;
  std::vector<int> treecodes;
  std::vector<int> litlenlengths;
  std::vector<int> distlengths;
  std::vector<int> lz77_lcode;
  std::vector<int> lz77_dcode;
  std::vector<int> lz77_lbits;
  std::vector<int> lz77_dbits;
  std::vector<int> lz77_lvalue;
  std::vector<int> lz77_dvalue;
  size_t numlit;
  size_t numlen;
};
void extractZlibInfo(std::vector<ZlibBlockInfo>& zlibinfo, const std::vector<unsigned char>& in);
}
#endif
```

## File: src/zopfli/deflate.c
```cpp
#include "deflate.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include "blocksplitter.h"
#include "squeeze.h"
#include "symbols.h"
#include "tree.h"
static void AddBit(int bit,
                   unsigned char* bp, unsigned char** out, size_t* outsize) {
  if (*bp == 0) ZOPFLI_APPEND_DATA(0, out, outsize);
  (*out)[*outsize - 1] |= bit << *bp;
  *bp = (*bp + 1) & 7;
}
static void AddBits(unsigned symbol, unsigned length,
                    unsigned char* bp, unsigned char** out, size_t* outsize) {
  unsigned i;
  for (i = 0; i < length; i++) {
    unsigned bit = (symbol >> i) & 1;
    if (*bp == 0) ZOPFLI_APPEND_DATA(0, out, outsize);
    (*out)[*outsize - 1] |= bit << *bp;
    *bp = (*bp + 1) & 7;
  }
}
static void AddHuffmanBits(unsigned symbol, unsigned length,
                           unsigned char* bp, unsigned char** out,
                           size_t* outsize) {
  unsigned i;
  for (i = 0; i < length; i++) {
    unsigned bit = (symbol >> (length - i - 1)) & 1;
    if (*bp == 0) ZOPFLI_APPEND_DATA(0, out, outsize);
    (*out)[*outsize - 1] |= bit << *bp;
    *bp = (*bp + 1) & 7;
  }
}
static void PatchDistanceCodesForBuggyDecoders(unsigned* d_lengths) {
  int num_dist_codes = 0;
  int i;
  for (i = 0; i < 30 ; i++) {
    if (d_lengths[i]) num_dist_codes++;
    if (num_dist_codes >= 2) return;
  }
  if (num_dist_codes == 0) {
    d_lengths[0] = d_lengths[1] = 1;
  } else if (num_dist_codes == 1) {
    d_lengths[d_lengths[0] ? 1 : 0] = 1;
  }
}
static size_t EncodeTree(const unsigned* ll_lengths,
                         const unsigned* d_lengths,
                         int use_16, int use_17, int use_18,
                         unsigned char* bp,
                         unsigned char** out, size_t* outsize) {
  unsigned lld_total;
  unsigned* rle = 0;
  unsigned* rle_bits = 0;
  size_t rle_size = 0;
  size_t rle_bits_size = 0;
  unsigned hlit = 29;
  unsigned hdist = 29;
  unsigned hclen;
  unsigned hlit2;
  size_t i, j;
  size_t clcounts[19];
  unsigned clcl[19];
  unsigned clsymbols[19];
  static const unsigned order[19] = {
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
  };
  int size_only = !out;
  size_t result_size = 0;
  for(i = 0; i < 19; i++) clcounts[i] = 0;
  while (hlit > 0 && ll_lengths[257 + hlit - 1] == 0) hlit--;
  while (hdist > 0 && d_lengths[1 + hdist - 1] == 0) hdist--;
  hlit2 = hlit + 257;
  lld_total = hlit2 + hdist + 1;
  for (i = 0; i < lld_total; i++) {
    unsigned char symbol = i < hlit2 ? ll_lengths[i] : d_lengths[i - hlit2];
    unsigned count = 1;
    if(use_16 || (symbol == 0 && (use_17 || use_18))) {
      for (j = i + 1; j < lld_total && symbol ==
          (j < hlit2 ? ll_lengths[j] : d_lengths[j - hlit2]); j++) {
        count++;
      }
    }
    i += count - 1;
    if (symbol == 0 && count >= 3) {
      if (use_18) {
        while (count >= 11) {
          unsigned count2 = count > 138 ? 138 : count;
          if (!size_only) {
            ZOPFLI_APPEND_DATA(18, &rle, &rle_size);
            ZOPFLI_APPEND_DATA(count2 - 11, &rle_bits, &rle_bits_size);
          }
          clcounts[18]++;
          count -= count2;
        }
      }
      if (use_17) {
        while (count >= 3) {
          unsigned count2 = count > 10 ? 10 : count;
          if (!size_only) {
            ZOPFLI_APPEND_DATA(17, &rle, &rle_size);
            ZOPFLI_APPEND_DATA(count2 - 3, &rle_bits, &rle_bits_size);
          }
          clcounts[17]++;
          count -= count2;
        }
      }
    }
    if (use_16 && count >= 4) {
      count--;
      clcounts[symbol]++;
      if (!size_only) {
        ZOPFLI_APPEND_DATA(symbol, &rle, &rle_size);
        ZOPFLI_APPEND_DATA(0, &rle_bits, &rle_bits_size);
      }
      while (count >= 3) {
        unsigned count2 = count > 6 ? 6 : count;
        if (!size_only) {
          ZOPFLI_APPEND_DATA(16, &rle, &rle_size);
          ZOPFLI_APPEND_DATA(count2 - 3, &rle_bits, &rle_bits_size);
        }
        clcounts[16]++;
        count -= count2;
      }
    }
    clcounts[symbol] += count;
    while (count > 0) {
      if (!size_only) {
        ZOPFLI_APPEND_DATA(symbol, &rle, &rle_size);
        ZOPFLI_APPEND_DATA(0, &rle_bits, &rle_bits_size);
      }
      count--;
    }
  }
  ZopfliCalculateBitLengths(clcounts, 19, 7, clcl);
  if (!size_only) ZopfliLengthsToSymbols(clcl, 19, 7, clsymbols);
  hclen = 15;
  while (hclen > 0 && clcounts[order[hclen + 4 - 1]] == 0) hclen--;
  if (!size_only) {
    AddBits(hlit, 5, bp, out, outsize);
    AddBits(hdist, 5, bp, out, outsize);
    AddBits(hclen, 4, bp, out, outsize);
    for (i = 0; i < hclen + 4; i++) {
      AddBits(clcl[order[i]], 3, bp, out, outsize);
    }
    for (i = 0; i < rle_size; i++) {
      unsigned symbol = clsymbols[rle[i]];
      AddHuffmanBits(symbol, clcl[rle[i]], bp, out, outsize);
      if (rle[i] == 16) AddBits(rle_bits[i], 2, bp, out, outsize);
      else if (rle[i] == 17) AddBits(rle_bits[i], 3, bp, out, outsize);
      else if (rle[i] == 18) AddBits(rle_bits[i], 7, bp, out, outsize);
    }
  }
  result_size += 14;
  result_size += (hclen + 4) * 3;
  for(i = 0; i < 19; i++) {
    result_size += clcl[i] * clcounts[i];
  }
  result_size += clcounts[16] * 2;
  result_size += clcounts[17] * 3;
  result_size += clcounts[18] * 7;
  free(rle);
  free(rle_bits);
  return result_size;
}
static void AddDynamicTree(const unsigned* ll_lengths,
                           const unsigned* d_lengths,
                           unsigned char* bp,
                           unsigned char** out, size_t* outsize) {
  int i;
  int best = 0;
  size_t bestsize = 0;
  for(i = 0; i < 8; i++) {
    size_t size = EncodeTree(ll_lengths, d_lengths,
                             i & 1, i & 2, i & 4,
                             0, 0, 0);
    if (bestsize == 0 || size < bestsize) {
      bestsize = size;
      best = i;
    }
  }
  EncodeTree(ll_lengths, d_lengths,
             best & 1, best & 2, best & 4,
             bp, out, outsize);
}
static size_t CalculateTreeSize(const unsigned* ll_lengths,
                                const unsigned* d_lengths) {
  size_t result = 0;
  int i;
  for(i = 0; i < 8; i++) {
    size_t size = EncodeTree(ll_lengths, d_lengths,
                             i & 1, i & 2, i & 4,
                             0, 0, 0);
    if (result == 0 || size < result) result = size;
  }
  return result;
}
static void AddLZ77Data(const ZopfliLZ77Store* lz77,
                        size_t lstart, size_t lend,
                        size_t expected_data_size,
                        const unsigned* ll_symbols, const unsigned* ll_lengths,
                        const unsigned* d_symbols, const unsigned* d_lengths,
                        unsigned char* bp,
                        unsigned char** out, size_t* outsize) {
  size_t testlength = 0;
  size_t i;
  for (i = lstart; i < lend; i++) {
    unsigned dist = lz77->dists[i];
    unsigned litlen = lz77->litlens[i];
    if (dist == 0) {
      assert(litlen < 256);
      assert(ll_lengths[litlen] > 0);
      AddHuffmanBits(ll_symbols[litlen], ll_lengths[litlen], bp, out, outsize);
      testlength++;
    } else {
      unsigned lls = ZopfliGetLengthSymbol(litlen);
      unsigned ds = ZopfliGetDistSymbol(dist);
      assert(litlen >= 3 && litlen <= 288);
      assert(ll_lengths[lls] > 0);
      assert(d_lengths[ds] > 0);
      AddHuffmanBits(ll_symbols[lls], ll_lengths[lls], bp, out, outsize);
      AddBits(ZopfliGetLengthExtraBitsValue(litlen),
              ZopfliGetLengthExtraBits(litlen),
              bp, out, outsize);
      AddHuffmanBits(d_symbols[ds], d_lengths[ds], bp, out, outsize);
      AddBits(ZopfliGetDistExtraBitsValue(dist),
              ZopfliGetDistExtraBits(dist),
              bp, out, outsize);
      testlength += litlen;
    }
  }
  assert(expected_data_size == 0 || testlength == expected_data_size);
}
static void GetFixedTree(unsigned* ll_lengths, unsigned* d_lengths) {
  size_t i;
  for (i = 0; i < 144; i++) ll_lengths[i] = 8;
  for (i = 144; i < 256; i++) ll_lengths[i] = 9;
  for (i = 256; i < 280; i++) ll_lengths[i] = 7;
  for (i = 280; i < 288; i++) ll_lengths[i] = 8;
  for (i = 0; i < 32; i++) d_lengths[i] = 5;
}
static size_t CalculateBlockSymbolSizeSmall(const unsigned* ll_lengths,
                                            const unsigned* d_lengths,
                                            const ZopfliLZ77Store* lz77,
                                            size_t lstart, size_t lend) {
  size_t result = 0;
  size_t i;
  for (i = lstart; i < lend; i++) {
    assert(i < lz77->size);
    assert(lz77->litlens[i] < 259);
    if (lz77->dists[i] == 0) {
      result += ll_lengths[lz77->litlens[i]];
    } else {
      int ll_symbol = ZopfliGetLengthSymbol(lz77->litlens[i]);
      int d_symbol = ZopfliGetDistSymbol(lz77->dists[i]);
      result += ll_lengths[ll_symbol];
      result += d_lengths[d_symbol];
      result += ZopfliGetLengthSymbolExtraBits(ll_symbol);
      result += ZopfliGetDistSymbolExtraBits(d_symbol);
    }
  }
  result += ll_lengths[256];
  return result;
}
static size_t CalculateBlockSymbolSizeGivenCounts(const size_t* ll_counts,
                                                  const size_t* d_counts,
                                                  const unsigned* ll_lengths,
                                                  const unsigned* d_lengths,
                                                  const ZopfliLZ77Store* lz77,
                                                  size_t lstart, size_t lend) {
  size_t result = 0;
  size_t i;
  if (lstart + ZOPFLI_NUM_LL * 3 > lend) {
    return CalculateBlockSymbolSizeSmall(
        ll_lengths, d_lengths, lz77, lstart, lend);
  } else {
    for (i = 0; i < 256; i++) {
      result += ll_lengths[i] * ll_counts[i];
    }
    for (i = 257; i < 286; i++) {
      result += ll_lengths[i] * ll_counts[i];
      result += ZopfliGetLengthSymbolExtraBits(i) * ll_counts[i];
    }
    for (i = 0; i < 30; i++) {
      result += d_lengths[i] * d_counts[i];
      result += ZopfliGetDistSymbolExtraBits(i) * d_counts[i];
    }
    result += ll_lengths[256];
    return result;
  }
}
static size_t CalculateBlockSymbolSize(const unsigned* ll_lengths,
                                       const unsigned* d_lengths,
                                       const ZopfliLZ77Store* lz77,
                                       size_t lstart, size_t lend) {
  if (lstart + ZOPFLI_NUM_LL * 3 > lend) {
    return CalculateBlockSymbolSizeSmall(
        ll_lengths, d_lengths, lz77, lstart, lend);
  } else {
    size_t ll_counts[ZOPFLI_NUM_LL];
    size_t d_counts[ZOPFLI_NUM_D];
    ZopfliLZ77GetHistogram(lz77, lstart, lend, ll_counts, d_counts);
    return CalculateBlockSymbolSizeGivenCounts(
        ll_counts, d_counts, ll_lengths, d_lengths, lz77, lstart, lend);
  }
}
static size_t AbsDiff(size_t x, size_t y) {
  if (x > y)
    return x - y;
  else
    return y - x;
}
void OptimizeHuffmanForRle(int length, size_t* counts) {
  int i, k, stride;
  size_t symbol, sum, limit;
  int* good_for_rle;
  for (; length >= 0; --length) {
    if (length == 0) {
      return;
    }
    if (counts[length - 1] != 0) {
      break;
    }
  }
  good_for_rle = (int*)malloc((unsigned)length * sizeof(int));
  for (i = 0; i < length; ++i) good_for_rle[i] = 0;
  symbol = counts[0];
  stride = 0;
  for (i = 0; i < length + 1; ++i) {
    if (i == length || counts[i] != symbol) {
      if ((symbol == 0 && stride >= 5) || (symbol != 0 && stride >= 7)) {
        for (k = 0; k < stride; ++k) {
          good_for_rle[i - k - 1] = 1;
        }
      }
      stride = 1;
      if (i != length) {
        symbol = counts[i];
      }
    } else {
      ++stride;
    }
  }
  stride = 0;
  limit = counts[0];
  sum = 0;
  for (i = 0; i < length + 1; ++i) {
    if (i == length || good_for_rle[i]
        || AbsDiff(counts[i], limit) >= 4) {
      if (stride >= 4 || (stride >= 3 && sum == 0)) {
        int count = (sum + stride / 2) / stride;
        if (count < 1) count = 1;
        if (sum == 0) {
          count = 0;
        }
        for (k = 0; k < stride; ++k) {
          counts[i - k - 1] = count;
        }
      }
      stride = 0;
      sum = 0;
      if (i < length - 3) {
        limit = (counts[i] + counts[i + 1] +
                 counts[i + 2] + counts[i + 3] + 2) / 4;
      } else if (i < length) {
        limit = counts[i];
      } else {
        limit = 0;
      }
    }
    ++stride;
    if (i != length) {
      sum += counts[i];
    }
  }
  free(good_for_rle);
}
static double TryOptimizeHuffmanForRle(
    const ZopfliLZ77Store* lz77, size_t lstart, size_t lend,
    const size_t* ll_counts, const size_t* d_counts,
    unsigned* ll_lengths, unsigned* d_lengths) {
  size_t ll_counts2[ZOPFLI_NUM_LL];
  size_t d_counts2[ZOPFLI_NUM_D];
  unsigned ll_lengths2[ZOPFLI_NUM_LL];
  unsigned d_lengths2[ZOPFLI_NUM_D];
  double treesize;
  double datasize;
  double treesize2;
  double datasize2;
  treesize = CalculateTreeSize(ll_lengths, d_lengths);
  datasize = CalculateBlockSymbolSizeGivenCounts(ll_counts, d_counts,
      ll_lengths, d_lengths, lz77, lstart, lend);
  memcpy(ll_counts2, ll_counts, sizeof(ll_counts2));
  memcpy(d_counts2, d_counts, sizeof(d_counts2));
  OptimizeHuffmanForRle(ZOPFLI_NUM_LL, ll_counts2);
  OptimizeHuffmanForRle(ZOPFLI_NUM_D, d_counts2);
  ZopfliCalculateBitLengths(ll_counts2, ZOPFLI_NUM_LL, 15, ll_lengths2);
  ZopfliCalculateBitLengths(d_counts2, ZOPFLI_NUM_D, 15, d_lengths2);
  PatchDistanceCodesForBuggyDecoders(d_lengths2);
  treesize2 = CalculateTreeSize(ll_lengths2, d_lengths2);
  datasize2 = CalculateBlockSymbolSizeGivenCounts(ll_counts, d_counts,
      ll_lengths2, d_lengths2, lz77, lstart, lend);
  if (treesize2 + datasize2 < treesize + datasize) {
    memcpy(ll_lengths, ll_lengths2, sizeof(ll_lengths2));
    memcpy(d_lengths, d_lengths2, sizeof(d_lengths2));
    return treesize2 + datasize2;
  }
  return treesize + datasize;
}
static double GetDynamicLengths(const ZopfliLZ77Store* lz77,
                                size_t lstart, size_t lend,
                                unsigned* ll_lengths, unsigned* d_lengths) {
  size_t ll_counts[ZOPFLI_NUM_LL];
  size_t d_counts[ZOPFLI_NUM_D];
  ZopfliLZ77GetHistogram(lz77, lstart, lend, ll_counts, d_counts);
  ll_counts[256] = 1;
  ZopfliCalculateBitLengths(ll_counts, ZOPFLI_NUM_LL, 15, ll_lengths);
  ZopfliCalculateBitLengths(d_counts, ZOPFLI_NUM_D, 15, d_lengths);
  PatchDistanceCodesForBuggyDecoders(d_lengths);
  return TryOptimizeHuffmanForRle(
      lz77, lstart, lend, ll_counts, d_counts, ll_lengths, d_lengths);
}
double ZopfliCalculateBlockSize(const ZopfliLZ77Store* lz77,
                                size_t lstart, size_t lend, int btype) {
  unsigned ll_lengths[ZOPFLI_NUM_LL];
  unsigned d_lengths[ZOPFLI_NUM_D];
  double result = 3;
  if (btype == 0) {
    size_t length = ZopfliLZ77GetByteRange(lz77, lstart, lend);
    size_t rem = length % 65535;
    size_t blocks = length / 65535 + (rem ? 1 : 0);
    return blocks * 5 * 8 + length * 8;
  } if (btype == 1) {
    GetFixedTree(ll_lengths, d_lengths);
    result += CalculateBlockSymbolSize(
        ll_lengths, d_lengths, lz77, lstart, lend);
  } else {
    result += GetDynamicLengths(lz77, lstart, lend, ll_lengths, d_lengths);
  }
  return result;
}
double ZopfliCalculateBlockSizeAutoType(const ZopfliLZ77Store* lz77,
                                        size_t lstart, size_t lend) {
  double uncompressedcost = ZopfliCalculateBlockSize(lz77, lstart, lend, 0);
  double fixedcost = (lz77->size > 1000) ?
      uncompressedcost : ZopfliCalculateBlockSize(lz77, lstart, lend, 1);
  double dyncost = ZopfliCalculateBlockSize(lz77, lstart, lend, 2);
  return (uncompressedcost < fixedcost && uncompressedcost < dyncost)
      ? uncompressedcost
      : (fixedcost < dyncost ? fixedcost : dyncost);
}
static void AddNonCompressedBlock(const ZopfliOptions* options, int final,
                                  const unsigned char* in, size_t instart,
                                  size_t inend,
                                  unsigned char* bp,
                                  unsigned char** out, size_t* outsize) {
  size_t pos = instart;
  (void)options;
  for (;;) {
    size_t i;
    unsigned short blocksize = 65535;
    unsigned short nlen;
    int currentfinal;
    if (pos + blocksize > inend) blocksize = inend - pos;
    currentfinal = pos + blocksize >= inend;
    nlen = ~blocksize;
    AddBit(final && currentfinal, bp, out, outsize);
    AddBit(0, bp, out, outsize);
    AddBit(0, bp, out, outsize);
    *bp = 0;
    ZOPFLI_APPEND_DATA(blocksize % 256, out, outsize);
    ZOPFLI_APPEND_DATA((blocksize / 256) % 256, out, outsize);
    ZOPFLI_APPEND_DATA(nlen % 256, out, outsize);
    ZOPFLI_APPEND_DATA((nlen / 256) % 256, out, outsize);
    for (i = 0; i < blocksize; i++) {
      ZOPFLI_APPEND_DATA(in[pos + i], out, outsize);
    }
    if (currentfinal) break;
    pos += blocksize;
  }
}
static void AddLZ77Block(const ZopfliOptions* options, int btype, int final,
                         const ZopfliLZ77Store* lz77,
                         size_t lstart, size_t lend,
                         size_t expected_data_size,
                         unsigned char* bp,
                         unsigned char** out, size_t* outsize) {
  unsigned ll_lengths[ZOPFLI_NUM_LL];
  unsigned d_lengths[ZOPFLI_NUM_D];
  unsigned ll_symbols[ZOPFLI_NUM_LL];
  unsigned d_symbols[ZOPFLI_NUM_D];
  size_t detect_block_size = *outsize;
  size_t compressed_size;
  size_t uncompressed_size = 0;
  size_t i;
  if (btype == 0) {
    size_t length = ZopfliLZ77GetByteRange(lz77, lstart, lend);
    size_t pos = lstart == lend ? 0 : lz77->pos[lstart];
    size_t end = pos + length;
    AddNonCompressedBlock(options, final,
                          lz77->data, pos, end, bp, out, outsize);
    return;
  }
  AddBit(final, bp, out, outsize);
  AddBit(btype & 1, bp, out, outsize);
  AddBit((btype & 2) >> 1, bp, out, outsize);
  if (btype == 1) {
    GetFixedTree(ll_lengths, d_lengths);
  } else {
    unsigned detect_tree_size;
    assert(btype == 2);
    GetDynamicLengths(lz77, lstart, lend, ll_lengths, d_lengths);
    detect_tree_size = *outsize;
    AddDynamicTree(ll_lengths, d_lengths, bp, out, outsize);
    if (options->verbose) {
      fprintf(stderr, "treesize: %d\n", (int)(*outsize - detect_tree_size));
    }
  }
  ZopfliLengthsToSymbols(ll_lengths, ZOPFLI_NUM_LL, 15, ll_symbols);
  ZopfliLengthsToSymbols(d_lengths, ZOPFLI_NUM_D, 15, d_symbols);
  detect_block_size = *outsize;
  AddLZ77Data(lz77, lstart, lend, expected_data_size,
              ll_symbols, ll_lengths, d_symbols, d_lengths,
              bp, out, outsize);
  AddHuffmanBits(ll_symbols[256], ll_lengths[256], bp, out, outsize);
  for (i = lstart; i < lend; i++) {
    uncompressed_size += lz77->dists[i] == 0 ? 1 : lz77->litlens[i];
  }
  compressed_size = *outsize - detect_block_size;
  if (options->verbose) {
    fprintf(stderr, "compressed block size: %d (%dk) (unc: %d)\n",
           (int)compressed_size, (int)(compressed_size / 1024),
           (int)(uncompressed_size));
  }
}
static void AddLZ77BlockAutoType(const ZopfliOptions* options, int final,
                                 const ZopfliLZ77Store* lz77,
                                 size_t lstart, size_t lend,
                                 size_t expected_data_size,
                                 unsigned char* bp,
                                 unsigned char** out, size_t* outsize) {
  double uncompressedcost = ZopfliCalculateBlockSize(lz77, lstart, lend, 0);
  double fixedcost = ZopfliCalculateBlockSize(lz77, lstart, lend, 1);
  double dyncost = ZopfliCalculateBlockSize(lz77, lstart, lend, 2);
  int expensivefixed = (lz77->size < 1000) || fixedcost <= dyncost * 1.1;
  ZopfliLZ77Store fixedstore;
  if (lstart == lend) {
    AddBits(final, 1, bp, out, outsize);
    AddBits(1, 2, bp, out, outsize);
    AddBits(0, 7, bp, out, outsize);
    return;
  }
  ZopfliInitLZ77Store(lz77->data, &fixedstore);
  if (expensivefixed) {
    size_t instart = lz77->pos[lstart];
    size_t inend = instart + ZopfliLZ77GetByteRange(lz77, lstart, lend);
    ZopfliBlockState s;
    ZopfliInitBlockState(options, instart, inend, 1, &s);
    ZopfliLZ77OptimalFixed(&s, lz77->data, instart, inend, &fixedstore);
    fixedcost = ZopfliCalculateBlockSize(&fixedstore, 0, fixedstore.size, 1);
    ZopfliCleanBlockState(&s);
  }
  if (uncompressedcost < fixedcost && uncompressedcost < dyncost) {
    AddLZ77Block(options, 0, final, lz77, lstart, lend,
                 expected_data_size, bp, out, outsize);
  } else if (fixedcost < dyncost) {
    if (expensivefixed) {
      AddLZ77Block(options, 1, final, &fixedstore, 0, fixedstore.size,
                   expected_data_size, bp, out, outsize);
    } else {
      AddLZ77Block(options, 1, final, lz77, lstart, lend,
                   expected_data_size, bp, out, outsize);
    }
  } else {
    AddLZ77Block(options, 2, final, lz77, lstart, lend,
                 expected_data_size, bp, out, outsize);
  }
  ZopfliCleanLZ77Store(&fixedstore);
}
void ZopfliDeflatePart(const ZopfliOptions* options, int btype, int final,
                       const unsigned char* in, size_t instart, size_t inend,
                       unsigned char* bp, unsigned char** out,
                       size_t* outsize) {
  size_t i;
  size_t* splitpoints_uncompressed = 0;
  size_t npoints = 0;
  size_t* splitpoints = 0;
  double totalcost = 0;
  ZopfliLZ77Store lz77;
  if (btype == 0) {
    AddNonCompressedBlock(options, final, in, instart, inend, bp, out, outsize);
    return;
  } else if (btype == 1) {
    ZopfliLZ77Store store;
    ZopfliBlockState s;
    ZopfliInitLZ77Store(in, &store);
    ZopfliInitBlockState(options, instart, inend, 1, &s);
    ZopfliLZ77OptimalFixed(&s, in, instart, inend, &store);
    AddLZ77Block(options, btype, final, &store, 0, store.size, 0,
                 bp, out, outsize);
    ZopfliCleanBlockState(&s);
    ZopfliCleanLZ77Store(&store);
    return;
  }
  if (options->blocksplitting) {
    ZopfliBlockSplit(options, in, instart, inend,
                     options->blocksplittingmax,
                     &splitpoints_uncompressed, &npoints);
    splitpoints = (size_t*)malloc(sizeof(*splitpoints) * npoints);
  }
  ZopfliInitLZ77Store(in, &lz77);
  for (i = 0; i <= npoints; i++) {
    size_t start = i == 0 ? instart : splitpoints_uncompressed[i - 1];
    size_t end = i == npoints ? inend : splitpoints_uncompressed[i];
    ZopfliBlockState s;
    ZopfliLZ77Store store;
    ZopfliInitLZ77Store(in, &store);
    ZopfliInitBlockState(options, start, end, 1, &s);
    ZopfliLZ77Optimal(&s, in, start, end, options->numiterations, &store);
    totalcost += ZopfliCalculateBlockSizeAutoType(&store, 0, store.size);
    ZopfliAppendLZ77Store(&store, &lz77);
    if (i < npoints) splitpoints[i] = lz77.size;
    ZopfliCleanBlockState(&s);
    ZopfliCleanLZ77Store(&store);
  }
  if (options->blocksplitting && npoints > 1) {
    size_t* splitpoints2 = 0;
    size_t npoints2 = 0;
    double totalcost2 = 0;
    ZopfliBlockSplitLZ77(options, &lz77,
                         options->blocksplittingmax, &splitpoints2, &npoints2);
    for (i = 0; i <= npoints2; i++) {
      size_t start = i == 0 ? 0 : splitpoints2[i - 1];
      size_t end = i == npoints2 ? lz77.size : splitpoints2[i];
      totalcost2 += ZopfliCalculateBlockSizeAutoType(&lz77, start, end);
    }
    if (totalcost2 < totalcost) {
      free(splitpoints);
      splitpoints = splitpoints2;
      npoints = npoints2;
    } else {
      free(splitpoints2);
    }
  }
  for (i = 0; i <= npoints; i++) {
    size_t start = i == 0 ? 0 : splitpoints[i - 1];
    size_t end = i == npoints ? lz77.size : splitpoints[i];
    AddLZ77BlockAutoType(options, i == npoints && final,
                         &lz77, start, end, 0,
                         bp, out, outsize);
  }
  ZopfliCleanLZ77Store(&lz77);
  free(splitpoints);
  free(splitpoints_uncompressed);
}
void ZopfliDeflate(const ZopfliOptions* options, int btype, int final,
                   const unsigned char* in, size_t insize,
                   unsigned char* bp, unsigned char** out, size_t* outsize) {
 size_t offset = *outsize;
#if ZOPFLI_MASTER_BLOCK_SIZE == 0
  ZopfliDeflatePart(options, btype, final, in, 0, insize, bp, out, outsize);
#else
  size_t i = 0;
  do {
    int masterfinal = (i + ZOPFLI_MASTER_BLOCK_SIZE >= insize);
    int final2 = final && masterfinal;
    size_t size = masterfinal ? insize - i : ZOPFLI_MASTER_BLOCK_SIZE;
    ZopfliDeflatePart(options, btype, final2,
                      in, i, i + size, bp, out, outsize);
    i += size;
  } while (i < insize);
#endif
  if (options->verbose) {
    fprintf(stderr,
            "Original Size: %lu, Deflate: %lu, Compression: %f%% Removed\n",
            (unsigned long)insize, (unsigned long)(*outsize - offset),
            100.0 * (double)(insize - (*outsize - offset)) / (double)insize);
  }
}
```

## File: src/zopfli/squeeze.c
```cpp
#include "squeeze.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "blocksplitter.h"
#include "deflate.h"
#include "symbols.h"
#include "tree.h"
#include "util.h"
typedef struct SymbolStats {
  size_t litlens[ZOPFLI_NUM_LL];
  size_t dists[ZOPFLI_NUM_D];
  double ll_symbols[ZOPFLI_NUM_LL];
  double d_symbols[ZOPFLI_NUM_D];
} SymbolStats;
static void InitStats(SymbolStats* stats) {
  memset(stats->litlens, 0, ZOPFLI_NUM_LL * sizeof(stats->litlens[0]));
  memset(stats->dists, 0, ZOPFLI_NUM_D * sizeof(stats->dists[0]));
  memset(stats->ll_symbols, 0, ZOPFLI_NUM_LL * sizeof(stats->ll_symbols[0]));
  memset(stats->d_symbols, 0, ZOPFLI_NUM_D * sizeof(stats->d_symbols[0]));
}
static void CopyStats(SymbolStats* source, SymbolStats* dest) {
  memcpy(dest->litlens, source->litlens,
         ZOPFLI_NUM_LL * sizeof(dest->litlens[0]));
  memcpy(dest->dists, source->dists, ZOPFLI_NUM_D * sizeof(dest->dists[0]));
  memcpy(dest->ll_symbols, source->ll_symbols,
         ZOPFLI_NUM_LL * sizeof(dest->ll_symbols[0]));
  memcpy(dest->d_symbols, source->d_symbols,
         ZOPFLI_NUM_D * sizeof(dest->d_symbols[0]));
}
static void AddWeighedStatFreqs(const SymbolStats* stats1, double w1,
                                const SymbolStats* stats2, double w2,
                                SymbolStats* result) {
  size_t i;
  for (i = 0; i < ZOPFLI_NUM_LL; i++) {
    result->litlens[i] =
        (size_t) (stats1->litlens[i] * w1 + stats2->litlens[i] * w2);
  }
  for (i = 0; i < ZOPFLI_NUM_D; i++) {
    result->dists[i] =
        (size_t) (stats1->dists[i] * w1 + stats2->dists[i] * w2);
  }
  result->litlens[256] = 1;
}
typedef struct RanState {
  unsigned int m_w, m_z;
} RanState;
static void InitRanState(RanState* state) {
  state->m_w = 1;
  state->m_z = 2;
}
static unsigned int Ran(RanState* state) {
  state->m_z = 36969 * (state->m_z & 65535) + (state->m_z >> 16);
  state->m_w = 18000 * (state->m_w & 65535) + (state->m_w >> 16);
  return (state->m_z << 16) + state->m_w;
}
static void RandomizeFreqs(RanState* state, size_t* freqs, int n) {
  int i;
  for (i = 0; i < n; i++) {
    if ((Ran(state) >> 4) % 3 == 0) freqs[i] = freqs[Ran(state) % n];
  }
}
static void RandomizeStatFreqs(RanState* state, SymbolStats* stats) {
  RandomizeFreqs(state, stats->litlens, ZOPFLI_NUM_LL);
  RandomizeFreqs(state, stats->dists, ZOPFLI_NUM_D);
  stats->litlens[256] = 1;
}
static void ClearStatFreqs(SymbolStats* stats) {
  size_t i;
  for (i = 0; i < ZOPFLI_NUM_LL; i++) stats->litlens[i] = 0;
  for (i = 0; i < ZOPFLI_NUM_D; i++) stats->dists[i] = 0;
}
typedef double CostModelFun(unsigned litlen, unsigned dist, void* context);
static double GetCostFixed(unsigned litlen, unsigned dist, void* unused) {
  (void)unused;
  if (dist == 0) {
    if (litlen <= 143) return 8;
    else return 9;
  } else {
    int dbits = ZopfliGetDistExtraBits(dist);
    int lbits = ZopfliGetLengthExtraBits(litlen);
    int lsym = ZopfliGetLengthSymbol(litlen);
    int cost = 0;
    if (lsym <= 279) cost += 7;
    else cost += 8;
    cost += 5;
    return cost + dbits + lbits;
  }
}
static double GetCostStat(unsigned litlen, unsigned dist, void* context) {
  SymbolStats* stats = (SymbolStats*)context;
  if (dist == 0) {
    return stats->ll_symbols[litlen];
  } else {
    int lsym = ZopfliGetLengthSymbol(litlen);
    int lbits = ZopfliGetLengthExtraBits(litlen);
    int dsym = ZopfliGetDistSymbol(dist);
    int dbits = ZopfliGetDistExtraBits(dist);
    return lbits + dbits + stats->ll_symbols[lsym] + stats->d_symbols[dsym];
  }
}
static double GetCostModelMinCost(CostModelFun* costmodel, void* costcontext) {
  double mincost;
  int bestlength = 0;
  int bestdist = 0;
  int i;
  static const int dsymbols[30] = {
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513,
    769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577
  };
  mincost = ZOPFLI_LARGE_FLOAT;
  for (i = 3; i < 259; i++) {
    double c = costmodel(i, 1, costcontext);
    if (c < mincost) {
      bestlength = i;
      mincost = c;
    }
  }
  mincost = ZOPFLI_LARGE_FLOAT;
  for (i = 0; i < 30; i++) {
    double c = costmodel(3, dsymbols[i], costcontext);
    if (c < mincost) {
      bestdist = dsymbols[i];
      mincost = c;
    }
  }
  return costmodel(bestlength, bestdist, costcontext);
}
static size_t zopfli_min(size_t a, size_t b) {
  return a < b ? a : b;
}
static double GetBestLengths(ZopfliBlockState *s,
                             const unsigned char* in,
                             size_t instart, size_t inend,
                             CostModelFun* costmodel, void* costcontext,
                             unsigned short* length_array,
                             ZopfliHash* h, float* costs) {
  size_t blocksize = inend - instart;
  size_t i = 0, k, kend;
  unsigned short leng;
  unsigned short dist;
  unsigned short sublen[259];
  size_t windowstart = instart > ZOPFLI_WINDOW_SIZE
      ? instart - ZOPFLI_WINDOW_SIZE : 0;
  double result;
  double mincost = GetCostModelMinCost(costmodel, costcontext);
  double mincostaddcostj;
  if (instart == inend) return 0;
  ZopfliResetHash(ZOPFLI_WINDOW_SIZE, h);
  ZopfliWarmupHash(in, windowstart, inend, h);
  for (i = windowstart; i < instart; i++) {
    ZopfliUpdateHash(in, i, inend, h);
  }
  for (i = 1; i < blocksize + 1; i++) costs[i] = ZOPFLI_LARGE_FLOAT;
  costs[0] = 0;
  length_array[0] = 0;
  for (i = instart; i < inend; i++) {
    size_t j = i - instart;
    ZopfliUpdateHash(in, i, inend, h);
#ifdef ZOPFLI_SHORTCUT_LONG_REPETITIONS
    if (h->same[i & ZOPFLI_WINDOW_MASK] > ZOPFLI_MAX_MATCH * 2
        && i > instart + ZOPFLI_MAX_MATCH + 1
        && i + ZOPFLI_MAX_MATCH * 2 + 1 < inend
        && h->same[(i - ZOPFLI_MAX_MATCH) & ZOPFLI_WINDOW_MASK]
            > ZOPFLI_MAX_MATCH) {
      double symbolcost = costmodel(ZOPFLI_MAX_MATCH, 1, costcontext);
      for (k = 0; k < ZOPFLI_MAX_MATCH; k++) {
        costs[j + ZOPFLI_MAX_MATCH] = costs[j] + symbolcost;
        length_array[j + ZOPFLI_MAX_MATCH] = ZOPFLI_MAX_MATCH;
        i++;
        j++;
        ZopfliUpdateHash(in, i, inend, h);
      }
    }
#endif
    ZopfliFindLongestMatch(s, h, in, i, inend, ZOPFLI_MAX_MATCH, sublen,
                           &dist, &leng);
    if (i + 1 <= inend) {
      double newCost = costmodel(in[i], 0, costcontext) + costs[j];
      assert(newCost >= 0);
      if (newCost < costs[j + 1]) {
        costs[j + 1] = newCost;
        length_array[j + 1] = 1;
      }
    }
    kend = zopfli_min(leng, inend-i);
    mincostaddcostj = mincost + costs[j];
    for (k = 3; k <= kend; k++) {
      double newCost;
     if (costs[j + k] <= mincostaddcostj) continue;
      newCost = costmodel(k, sublen[k], costcontext) + costs[j];
      assert(newCost >= 0);
      if (newCost < costs[j + k]) {
        assert(k <= ZOPFLI_MAX_MATCH);
        costs[j + k] = newCost;
        length_array[j + k] = k;
      }
    }
  }
  assert(costs[blocksize] >= 0);
  result = costs[blocksize];
  return result;
}
static void TraceBackwards(size_t size, const unsigned short* length_array,
                           unsigned short** path, size_t* pathsize) {
  size_t index = size;
  if (size == 0) return;
  for (;;) {
    ZOPFLI_APPEND_DATA(length_array[index], path, pathsize);
    assert(length_array[index] <= index);
    assert(length_array[index] <= ZOPFLI_MAX_MATCH);
    assert(length_array[index] != 0);
    index -= length_array[index];
    if (index == 0) break;
  }
  for (index = 0; index < *pathsize / 2; index++) {
    unsigned short temp = (*path)[index];
    (*path)[index] = (*path)[*pathsize - index - 1];
    (*path)[*pathsize - index - 1] = temp;
  }
}
static void FollowPath(ZopfliBlockState* s,
                       const unsigned char* in, size_t instart, size_t inend,
                       unsigned short* path, size_t pathsize,
                       ZopfliLZ77Store* store, ZopfliHash *h) {
  size_t i, j, pos = 0;
  size_t windowstart = instart > ZOPFLI_WINDOW_SIZE
      ? instart - ZOPFLI_WINDOW_SIZE : 0;
  size_t total_length_test = 0;
  if (instart == inend) return;
  ZopfliResetHash(ZOPFLI_WINDOW_SIZE, h);
  ZopfliWarmupHash(in, windowstart, inend, h);
  for (i = windowstart; i < instart; i++) {
    ZopfliUpdateHash(in, i, inend, h);
  }
  pos = instart;
  for (i = 0; i < pathsize; i++) {
    unsigned short length = path[i];
    unsigned short dummy_length;
    unsigned short dist;
    assert(pos < inend);
    ZopfliUpdateHash(in, pos, inend, h);
    if (length >= ZOPFLI_MIN_MATCH) {
      ZopfliFindLongestMatch(s, h, in, pos, inend, length, 0,
                             &dist, &dummy_length);
      assert(!(dummy_length != length && length > 2 && dummy_length > 2));
      ZopfliVerifyLenDist(in, inend, pos, dist, length);
      ZopfliStoreLitLenDist(length, dist, pos, store);
      total_length_test += length;
    } else {
      length = 1;
      ZopfliStoreLitLenDist(in[pos], 0, pos, store);
      total_length_test++;
    }
    assert(pos + length <= inend);
    for (j = 1; j < length; j++) {
      ZopfliUpdateHash(in, pos + j, inend, h);
    }
    pos += length;
  }
}
static void CalculateStatistics(SymbolStats* stats) {
  ZopfliCalculateEntropy(stats->litlens, ZOPFLI_NUM_LL, stats->ll_symbols);
  ZopfliCalculateEntropy(stats->dists, ZOPFLI_NUM_D, stats->d_symbols);
}
static void GetStatistics(const ZopfliLZ77Store* store, SymbolStats* stats) {
  size_t i;
  for (i = 0; i < store->size; i++) {
    if (store->dists[i] == 0) {
      stats->litlens[store->litlens[i]]++;
    } else {
      stats->litlens[ZopfliGetLengthSymbol(store->litlens[i])]++;
      stats->dists[ZopfliGetDistSymbol(store->dists[i])]++;
    }
  }
  stats->litlens[256] = 1;
  CalculateStatistics(stats);
}
static double LZ77OptimalRun(ZopfliBlockState* s,
    const unsigned char* in, size_t instart, size_t inend,
    unsigned short** path, size_t* pathsize,
    unsigned short* length_array, CostModelFun* costmodel,
    void* costcontext, ZopfliLZ77Store* store,
    ZopfliHash* h, float* costs) {
  double cost = GetBestLengths(s, in, instart, inend, costmodel,
                costcontext, length_array, h, costs);
  free(*path);
  *path = 0;
  *pathsize = 0;
  TraceBackwards(inend - instart, length_array, path, pathsize);
  FollowPath(s, in, instart, inend, *path, *pathsize, store, h);
  assert(cost < ZOPFLI_LARGE_FLOAT);
  return cost;
}
void ZopfliLZ77Optimal(ZopfliBlockState *s,
                       const unsigned char* in, size_t instart, size_t inend,
                       int numiterations,
                       ZopfliLZ77Store* store) {
  size_t blocksize = inend - instart;
  unsigned short* length_array =
      (unsigned short*)malloc(sizeof(unsigned short) * (blocksize + 1));
  unsigned short* path = 0;
  size_t pathsize = 0;
  ZopfliLZ77Store currentstore;
  ZopfliHash hash;
  ZopfliHash* h = &hash;
  SymbolStats stats, beststats, laststats;
  int i;
  float* costs = (float*)malloc(sizeof(float) * (blocksize + 1));
  double cost;
  double bestcost = ZOPFLI_LARGE_FLOAT;
  double lastcost = 0;
  RanState ran_state;
  int lastrandomstep = -1;
  if (!costs) exit(-1);
  if (!length_array) exit(-1);
  InitRanState(&ran_state);
  InitStats(&stats);
  ZopfliInitLZ77Store(in, &currentstore);
  ZopfliAllocHash(ZOPFLI_WINDOW_SIZE, h);
  ZopfliLZ77Greedy(s, in, instart, inend, &currentstore, h);
  GetStatistics(&currentstore, &stats);
  for (i = 0; i < numiterations; i++) {
    ZopfliCleanLZ77Store(&currentstore);
    ZopfliInitLZ77Store(in, &currentstore);
    LZ77OptimalRun(s, in, instart, inend, &path, &pathsize,
                   length_array, GetCostStat, (void*)&stats,
                   &currentstore, h, costs);
    cost = ZopfliCalculateBlockSize(&currentstore, 0, currentstore.size, 2);
    if (s->options->verbose_more || (s->options->verbose && cost < bestcost)) {
      fprintf(stderr, "Iteration %d: %d bit\n", i, (int) cost);
    }
    if (cost < bestcost) {
      ZopfliCopyLZ77Store(&currentstore, store);
      CopyStats(&stats, &beststats);
      bestcost = cost;
    }
    CopyStats(&stats, &laststats);
    ClearStatFreqs(&stats);
    GetStatistics(&currentstore, &stats);
    if (lastrandomstep != -1) {
      AddWeighedStatFreqs(&stats, 1.0, &laststats, 0.5, &stats);
      CalculateStatistics(&stats);
    }
    if (i > 5 && cost == lastcost) {
      CopyStats(&beststats, &stats);
      RandomizeStatFreqs(&ran_state, &stats);
      CalculateStatistics(&stats);
      lastrandomstep = i;
    }
    lastcost = cost;
  }
  free(length_array);
  free(path);
  free(costs);
  ZopfliCleanLZ77Store(&currentstore);
  ZopfliCleanHash(h);
}
void ZopfliLZ77OptimalFixed(ZopfliBlockState *s,
                            const unsigned char* in,
                            size_t instart, size_t inend,
                            ZopfliLZ77Store* store)
{
  size_t blocksize = inend - instart;
  unsigned short* length_array =
      (unsigned short*)malloc(sizeof(unsigned short) * (blocksize + 1));
  unsigned short* path = 0;
  size_t pathsize = 0;
  ZopfliHash hash;
  ZopfliHash* h = &hash;
  float* costs = (float*)malloc(sizeof(float) * (blocksize + 1));
  if (!costs) exit(-1);
  if (!length_array) exit(-1);
  ZopfliAllocHash(ZOPFLI_WINDOW_SIZE, h);
  s->blockstart = instart;
  s->blockend = inend;
  LZ77OptimalRun(s, in, instart, inend, &path, &pathsize,
                 length_array, GetCostFixed, 0, store, h, costs);
  free(length_array);
  free(path);
  free(costs);
  ZopfliCleanHash(h);
}
```

## File: src/zopflipng/lodepng/lodepng.h
```
#ifndef LODEPNG_H
#define LODEPNG_H
#include <string.h>
extern const char* LODEPNG_VERSION_STRING;
#ifndef LODEPNG_NO_COMPILE_ZLIB
#define LODEPNG_COMPILE_ZLIB
#endif
#ifndef LODEPNG_NO_COMPILE_PNG
#define LODEPNG_COMPILE_PNG
#endif
#ifndef LODEPNG_NO_COMPILE_DECODER
#define LODEPNG_COMPILE_DECODER
#endif
#ifndef LODEPNG_NO_COMPILE_ENCODER
#define LODEPNG_COMPILE_ENCODER
#endif
#ifndef LODEPNG_NO_COMPILE_DISK
#define LODEPNG_COMPILE_DISK
#endif
#ifndef LODEPNG_NO_COMPILE_ANCILLARY_CHUNKS
#define LODEPNG_COMPILE_ANCILLARY_CHUNKS
#endif
#ifndef LODEPNG_NO_COMPILE_ERROR_TEXT
#define LODEPNG_COMPILE_ERROR_TEXT
#endif
#ifndef LODEPNG_NO_COMPILE_ALLOCATORS
#define LODEPNG_COMPILE_ALLOCATORS
#endif
#ifdef __cplusplus
#ifndef LODEPNG_NO_COMPILE_CPP
#define LODEPNG_COMPILE_CPP
#endif
#endif
#ifdef LODEPNG_COMPILE_CPP
#include <vector>
#include <string>
#endif
#ifdef LODEPNG_COMPILE_PNG
typedef enum LodePNGColorType {
  LCT_GREY = 0,
  LCT_RGB = 2,
  LCT_PALETTE = 3,
  LCT_GREY_ALPHA = 4,
  LCT_RGBA = 6,
  LCT_MAX_OCTET_VALUE = 255
} LodePNGColorType;
#ifdef LODEPNG_COMPILE_DECODER
unsigned lodepng_decode_memory(unsigned char** out, unsigned* w, unsigned* h,
                               const unsigned char* in, size_t insize,
                               LodePNGColorType colortype, unsigned bitdepth);
unsigned lodepng_decode32(unsigned char** out, unsigned* w, unsigned* h,
                          const unsigned char* in, size_t insize);
unsigned lodepng_decode24(unsigned char** out, unsigned* w, unsigned* h,
                          const unsigned char* in, size_t insize);
#ifdef LODEPNG_COMPILE_DISK
unsigned lodepng_decode_file(unsigned char** out, unsigned* w, unsigned* h,
                             const char* filename,
                             LodePNGColorType colortype, unsigned bitdepth);
unsigned lodepng_decode32_file(unsigned char** out, unsigned* w, unsigned* h,
                               const char* filename);
unsigned lodepng_decode24_file(unsigned char** out, unsigned* w, unsigned* h,
                               const char* filename);
#endif
#endif
#ifdef LODEPNG_COMPILE_ENCODER
unsigned lodepng_encode_memory(unsigned char** out, size_t* outsize,
                               const unsigned char* image, unsigned w, unsigned h,
                               LodePNGColorType colortype, unsigned bitdepth);
unsigned lodepng_encode32(unsigned char** out, size_t* outsize,
                          const unsigned char* image, unsigned w, unsigned h);
unsigned lodepng_encode24(unsigned char** out, size_t* outsize,
                          const unsigned char* image, unsigned w, unsigned h);
#ifdef LODEPNG_COMPILE_DISK
unsigned lodepng_encode_file(const char* filename,
                             const unsigned char* image, unsigned w, unsigned h,
                             LodePNGColorType colortype, unsigned bitdepth);
unsigned lodepng_encode32_file(const char* filename,
                               const unsigned char* image, unsigned w, unsigned h);
unsigned lodepng_encode24_file(const char* filename,
                               const unsigned char* image, unsigned w, unsigned h);
#endif
#endif
#ifdef LODEPNG_COMPILE_CPP
namespace lodepng {
#ifdef LODEPNG_COMPILE_DECODER
unsigned decode(std::vector<unsigned char>& out, unsigned& w, unsigned& h,
                const unsigned char* in, size_t insize,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
unsigned decode(std::vector<unsigned char>& out, unsigned& w, unsigned& h,
                const std::vector<unsigned char>& in,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
#ifdef LODEPNG_COMPILE_DISK
unsigned decode(std::vector<unsigned char>& out, unsigned& w, unsigned& h,
                const std::string& filename,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
#endif
#endif
#ifdef LODEPNG_COMPILE_ENCODER
unsigned encode(std::vector<unsigned char>& out,
                const unsigned char* in, unsigned w, unsigned h,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
unsigned encode(std::vector<unsigned char>& out,
                const std::vector<unsigned char>& in, unsigned w, unsigned h,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
#ifdef LODEPNG_COMPILE_DISK
unsigned encode(const std::string& filename,
                const unsigned char* in, unsigned w, unsigned h,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
unsigned encode(const std::string& filename,
                const std::vector<unsigned char>& in, unsigned w, unsigned h,
                LodePNGColorType colortype = LCT_RGBA, unsigned bitdepth = 8);
#endif
#endif
}
#endif
#endif
#ifdef LODEPNG_COMPILE_ERROR_TEXT
const char* lodepng_error_text(unsigned code);
#endif
#ifdef LODEPNG_COMPILE_DECODER
typedef struct LodePNGDecompressSettings LodePNGDecompressSettings;
struct LodePNGDecompressSettings {
  unsigned ignore_adler32;
  unsigned ignore_nlen;
  size_t max_output_size;
  unsigned (*custom_zlib)(unsigned char**, size_t*,
                          const unsigned char*, size_t,
                          const LodePNGDecompressSettings*);
  unsigned (*custom_inflate)(unsigned char**, size_t*,
                             const unsigned char*, size_t,
                             const LodePNGDecompressSettings*);
  const void* custom_context;
};
extern const LodePNGDecompressSettings lodepng_default_decompress_settings;
void lodepng_decompress_settings_init(LodePNGDecompressSettings* settings);
#endif
#ifdef LODEPNG_COMPILE_ENCODER
typedef struct LodePNGCompressSettings LodePNGCompressSettings;
struct LodePNGCompressSettings  {
  unsigned btype;
  unsigned use_lz77;
  unsigned windowsize;
  unsigned minmatch;
  unsigned nicematch;
  unsigned lazymatching;
  unsigned (*custom_zlib)(unsigned char**, size_t*,
                          const unsigned char*, size_t,
                          const LodePNGCompressSettings*);
  unsigned (*custom_deflate)(unsigned char**, size_t*,
                             const unsigned char*, size_t,
                             const LodePNGCompressSettings*);
  const void* custom_context;
};
extern const LodePNGCompressSettings lodepng_default_compress_settings;
void lodepng_compress_settings_init(LodePNGCompressSettings* settings);
#endif
#ifdef LODEPNG_COMPILE_PNG
typedef struct LodePNGColorMode {
  LodePNGColorType colortype;
  unsigned bitdepth;
  unsigned char* palette;
  size_t palettesize;
  unsigned key_defined;
  unsigned key_r;
  unsigned key_g;
  unsigned key_b;
} LodePNGColorMode;
void lodepng_color_mode_init(LodePNGColorMode* info);
void lodepng_color_mode_cleanup(LodePNGColorMode* info);
unsigned lodepng_color_mode_copy(LodePNGColorMode* dest, const LodePNGColorMode* source);
LodePNGColorMode lodepng_color_mode_make(LodePNGColorType colortype, unsigned bitdepth);
void lodepng_palette_clear(LodePNGColorMode* info);
unsigned lodepng_palette_add(LodePNGColorMode* info,
                             unsigned char r, unsigned char g, unsigned char b, unsigned char a);
unsigned lodepng_get_bpp(const LodePNGColorMode* info);
unsigned lodepng_get_channels(const LodePNGColorMode* info);
unsigned lodepng_is_greyscale_type(const LodePNGColorMode* info);
unsigned lodepng_is_alpha_type(const LodePNGColorMode* info);
unsigned lodepng_is_palette_type(const LodePNGColorMode* info);
unsigned lodepng_has_palette_alpha(const LodePNGColorMode* info);
unsigned lodepng_can_have_alpha(const LodePNGColorMode* info);
size_t lodepng_get_raw_size(unsigned w, unsigned h, const LodePNGColorMode* color);
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
typedef struct LodePNGTime {
  unsigned year;
  unsigned month;
  unsigned day;
  unsigned hour;
  unsigned minute;
  unsigned second;
} LodePNGTime;
#endif
typedef struct LodePNGInfo {
  unsigned compression_method;
  unsigned filter_method;
  unsigned interlace_method;
  LodePNGColorMode color;
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
  unsigned background_defined;
  unsigned background_r;
  unsigned background_g;
  unsigned background_b;
  size_t text_num;
  char** text_keys;
  char** text_strings;
  size_t itext_num;
  char** itext_keys;
  char** itext_langtags;
  char** itext_transkeys;
  char** itext_strings;
  unsigned time_defined;
  LodePNGTime time;
  unsigned phys_defined;
  unsigned phys_x;
  unsigned phys_y;
  unsigned phys_unit;
  unsigned gama_defined;
  unsigned gama_gamma;
  unsigned chrm_defined;
  unsigned chrm_white_x;
  unsigned chrm_white_y;
  unsigned chrm_red_x;
  unsigned chrm_red_y;
  unsigned chrm_green_x;
  unsigned chrm_green_y;
  unsigned chrm_blue_x;
  unsigned chrm_blue_y;
  unsigned srgb_defined;
  unsigned srgb_intent;
  unsigned iccp_defined;
  char* iccp_name;
  unsigned char* iccp_profile;
  unsigned iccp_profile_size;
  unsigned char* unknown_chunks_data[3];
  size_t unknown_chunks_size[3];
#endif
} LodePNGInfo;
void lodepng_info_init(LodePNGInfo* info);
void lodepng_info_cleanup(LodePNGInfo* info);
unsigned lodepng_info_copy(LodePNGInfo* dest, const LodePNGInfo* source);
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
unsigned lodepng_add_text(LodePNGInfo* info, const char* key, const char* str);
void lodepng_clear_text(LodePNGInfo* info);
unsigned lodepng_add_itext(LodePNGInfo* info, const char* key, const char* langtag,
                           const char* transkey, const char* str);
void lodepng_clear_itext(LodePNGInfo* info);
unsigned lodepng_set_icc(LodePNGInfo* info, const char* name, const unsigned char* profile, unsigned profile_size);
void lodepng_clear_icc(LodePNGInfo* info);
#endif
unsigned lodepng_convert(unsigned char* out, const unsigned char* in,
                         const LodePNGColorMode* mode_out, const LodePNGColorMode* mode_in,
                         unsigned w, unsigned h);
#ifdef LODEPNG_COMPILE_DECODER
typedef struct LodePNGDecoderSettings {
  LodePNGDecompressSettings zlibsettings;
  unsigned ignore_crc;
  unsigned ignore_critical;
  unsigned ignore_end;
  unsigned color_convert;
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
  unsigned read_text_chunks;
  unsigned remember_unknown_chunks;
  size_t max_text_size;
  size_t max_icc_size;
#endif
} LodePNGDecoderSettings;
void lodepng_decoder_settings_init(LodePNGDecoderSettings* settings);
#endif
#ifdef LODEPNG_COMPILE_ENCODER
typedef enum LodePNGFilterStrategy {
  LFS_ZERO = 0,
  LFS_ONE = 1,
  LFS_TWO = 2,
  LFS_THREE = 3,
  LFS_FOUR = 4,
  LFS_MINSUM,
  LFS_ENTROPY,
  LFS_BRUTE_FORCE,
  LFS_PREDEFINED
} LodePNGFilterStrategy;
typedef struct LodePNGColorStats {
  unsigned colored;
  unsigned key;
  unsigned short key_r;
  unsigned short key_g;
  unsigned short key_b;
  unsigned alpha;
  unsigned numcolors;
  unsigned char palette[1024];
  unsigned bits;
  size_t numpixels;
  unsigned allow_palette;
  unsigned allow_greyscale;
} LodePNGColorStats;
void lodepng_color_stats_init(LodePNGColorStats* stats);
unsigned lodepng_compute_color_stats(LodePNGColorStats* stats,
                                     const unsigned char* image, unsigned w, unsigned h,
                                     const LodePNGColorMode* mode_in);
typedef struct LodePNGEncoderSettings {
  LodePNGCompressSettings zlibsettings;
  unsigned auto_convert;
  unsigned filter_palette_zero;
  LodePNGFilterStrategy filter_strategy;
  const unsigned char* predefined_filters;
  unsigned force_palette;
#ifdef LODEPNG_COMPILE_ANCILLARY_CHUNKS
  unsigned add_id;
  unsigned text_compression;
#endif
} LodePNGEncoderSettings;
void lodepng_encoder_settings_init(LodePNGEncoderSettings* settings);
#endif
#if defined(LODEPNG_COMPILE_DECODER) || defined(LODEPNG_COMPILE_ENCODER)
typedef struct LodePNGState {
#ifdef LODEPNG_COMPILE_DECODER
  LodePNGDecoderSettings decoder;
#endif
#ifdef LODEPNG_COMPILE_ENCODER
  LodePNGEncoderSettings encoder;
#endif
  LodePNGColorMode info_raw;
  LodePNGInfo info_png;
  unsigned error;
} LodePNGState;
void lodepng_state_init(LodePNGState* state);
void lodepng_state_cleanup(LodePNGState* state);
void lodepng_state_copy(LodePNGState* dest, const LodePNGState* source);
#endif
#ifdef LODEPNG_COMPILE_DECODER
unsigned lodepng_decode(unsigned char** out, unsigned* w, unsigned* h,
                        LodePNGState* state,
                        const unsigned char* in, size_t insize);
unsigned lodepng_inspect(unsigned* w, unsigned* h,
                         LodePNGState* state,
                         const unsigned char* in, size_t insize);
#endif
unsigned lodepng_inspect_chunk(LodePNGState* state, size_t pos,
                               const unsigned char* in, size_t insize);
#ifdef LODEPNG_COMPILE_ENCODER
unsigned lodepng_encode(unsigned char** out, size_t* outsize,
                        const unsigned char* image, unsigned w, unsigned h,
                        LodePNGState* state);
#endif
unsigned lodepng_chunk_length(const unsigned char* chunk);
void lodepng_chunk_type(char type[5], const unsigned char* chunk);
unsigned char lodepng_chunk_type_equals(const unsigned char* chunk, const char* type);
unsigned char lodepng_chunk_ancillary(const unsigned char* chunk);
unsigned char lodepng_chunk_private(const unsigned char* chunk);
unsigned char lodepng_chunk_safetocopy(const unsigned char* chunk);
unsigned char* lodepng_chunk_data(unsigned char* chunk);
const unsigned char* lodepng_chunk_data_const(const unsigned char* chunk);
unsigned lodepng_chunk_check_crc(const unsigned char* chunk);
void lodepng_chunk_generate_crc(unsigned char* chunk);
unsigned char* lodepng_chunk_next(unsigned char* chunk, unsigned char* end);
const unsigned char* lodepng_chunk_next_const(const unsigned char* chunk, const unsigned char* end);
unsigned char* lodepng_chunk_find(unsigned char* chunk, unsigned char* end, const char type[5]);
const unsigned char* lodepng_chunk_find_const(const unsigned char* chunk, const unsigned char* end, const char type[5]);
unsigned lodepng_chunk_append(unsigned char** out, size_t* outsize, const unsigned char* chunk);
unsigned lodepng_chunk_create(unsigned char** out, size_t* outsize, unsigned length,
                              const char* type, const unsigned char* data);
unsigned lodepng_crc32(const unsigned char* buf, size_t len);
#endif
#ifdef LODEPNG_COMPILE_ZLIB
#ifdef LODEPNG_COMPILE_DECODER
unsigned lodepng_inflate(unsigned char** out, size_t* outsize,
                         const unsigned char* in, size_t insize,
                         const LodePNGDecompressSettings* settings);
unsigned lodepng_zlib_decompress(unsigned char** out, size_t* outsize,
                                 const unsigned char* in, size_t insize,
                                 const LodePNGDecompressSettings* settings);
#endif
#ifdef LODEPNG_COMPILE_ENCODER
unsigned lodepng_zlib_compress(unsigned char** out, size_t* outsize,
                               const unsigned char* in, size_t insize,
                               const LodePNGCompressSettings* settings);
unsigned lodepng_huffman_code_lengths(unsigned* lengths, const unsigned* frequencies,
                                      size_t numcodes, unsigned maxbitlen);
unsigned lodepng_deflate(unsigned char** out, size_t* outsize,
                         const unsigned char* in, size_t insize,
                         const LodePNGCompressSettings* settings);
#endif
#endif
#ifdef LODEPNG_COMPILE_DISK
unsigned lodepng_load_file(unsigned char** out, size_t* outsize, const char* filename);
unsigned lodepng_save_file(const unsigned char* buffer, size_t buffersize, const char* filename);
#endif
#ifdef LODEPNG_COMPILE_CPP
namespace lodepng {
#ifdef LODEPNG_COMPILE_PNG
class State : public LodePNGState {
  public:
    State();
    State(const State& other);
    ~State();
    State& operator=(const State& other);
};
#ifdef LODEPNG_COMPILE_DECODER
unsigned decode(std::vector<unsigned char>& out, unsigned& w, unsigned& h,
                State& state,
                const unsigned char* in, size_t insize);
unsigned decode(std::vector<unsigned char>& out, unsigned& w, unsigned& h,
                State& state,
                const std::vector<unsigned char>& in);
#endif
#ifdef LODEPNG_COMPILE_ENCODER
unsigned encode(std::vector<unsigned char>& out,
                const unsigned char* in, unsigned w, unsigned h,
                State& state);
unsigned encode(std::vector<unsigned char>& out,
                const std::vector<unsigned char>& in, unsigned w, unsigned h,
                State& state);
#endif
#ifdef LODEPNG_COMPILE_DISK
unsigned load_file(std::vector<unsigned char>& buffer, const std::string& filename);
unsigned save_file(const std::vector<unsigned char>& buffer, const std::string& filename);
#endif
#endif
#ifdef LODEPNG_COMPILE_ZLIB
#ifdef LODEPNG_COMPILE_DECODER
unsigned decompress(std::vector<unsigned char>& out, const unsigned char* in, size_t insize,
                    const LodePNGDecompressSettings& settings = lodepng_default_decompress_settings);
unsigned decompress(std::vector<unsigned char>& out, const std::vector<unsigned char>& in,
                    const LodePNGDecompressSettings& settings = lodepng_default_decompress_settings);
#endif
#ifdef LODEPNG_COMPILE_ENCODER
unsigned compress(std::vector<unsigned char>& out, const unsigned char* in, size_t insize,
                  const LodePNGCompressSettings& settings = lodepng_default_compress_settings);
unsigned compress(std::vector<unsigned char>& out, const std::vector<unsigned char>& in,
                  const LodePNGCompressSettings& settings = lodepng_default_compress_settings);
#endif
#endif
}
#endif
#endif
```

## File: src/zopfli/zopfli_bin.c
```cpp
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "deflate.h"
#include "gzip_container.h"
#include "zlib_container.h"
#if _WIN32
#include <fcntl.h>
#endif
static int LoadFile(const char* filename,
                    unsigned char** out, size_t* outsize) {
  FILE* file;
  *out = 0;
  *outsize = 0;
  file = fopen(filename, "rb");
  if (!file) return 0;
  fseek(file , 0 , SEEK_END);
  *outsize = ftell(file);
  if(*outsize > 2147483647) {
    fprintf(stderr,"Files larger than 2GB are not supported.\n");
    exit(EXIT_FAILURE);
  }
  rewind(file);
  *out = (unsigned char*)malloc(*outsize);
  if (*outsize && (*out)) {
    size_t testsize = fread(*out, 1, *outsize, file);
    if (testsize != *outsize) {
      free(*out);
      *out = 0;
      *outsize = 0;
      fclose(file);
      return 0;
    }
  }
  assert(!(*outsize) || out);
  fclose(file);
  return 1;
}
static void SaveFile(const char* filename,
                     const unsigned char* in, size_t insize) {
  FILE* file = fopen(filename, "wb" );
  if (file == NULL) {
      fprintf(stderr,"Error: Cannot write to output file, terminating.\n");
      exit (EXIT_FAILURE);
  }
  assert(file);
  fwrite((char*)in, 1, insize, file);
  fclose(file);
}
static void CompressFile(const ZopfliOptions* options,
                         ZopfliFormat output_type,
                         const char* infilename,
                         const char* outfilename) {
  unsigned char* in;
  size_t insize;
  unsigned char* out = 0;
  size_t outsize = 0;
  if (!LoadFile(infilename, &in, &insize)) {
    fprintf(stderr, "Invalid filename: %s\n", infilename);
    return;
  }
  ZopfliCompress(options, output_type, in, insize, &out, &outsize);
  if (outfilename) {
    SaveFile(outfilename, out, outsize);
  } else {
#if _WIN32
    _setmode(_fileno(stdout), _O_BINARY);
#endif
    fwrite(out, 1, outsize, stdout);
  }
  free(out);
  free(in);
}
static char* AddStrings(const char* str1, const char* str2) {
  size_t len = strlen(str1) + strlen(str2);
  char* result = (char*)malloc(len + 1);
  if (!result) exit(-1);
  strcpy(result, str1);
  strcat(result, str2);
  return result;
}
static char StringsEqual(const char* str1, const char* str2) {
  return strcmp(str1, str2) == 0;
}
int main(int argc, char* argv[]) {
  ZopfliOptions options;
  ZopfliFormat output_type = ZOPFLI_FORMAT_GZIP;
  const char* filename = 0;
  int output_to_stdout = 0;
  int i;
  ZopfliInitOptions(&options);
  for (i = 1; i < argc; i++) {
    const char* arg = argv[i];
    if (StringsEqual(arg, "-v")) options.verbose = 1;
    else if (StringsEqual(arg, "-c")) output_to_stdout = 1;
    else if (StringsEqual(arg, "--deflate")) {
      output_type = ZOPFLI_FORMAT_DEFLATE;
    }
    else if (StringsEqual(arg, "--zlib")) output_type = ZOPFLI_FORMAT_ZLIB;
    else if (StringsEqual(arg, "--gzip")) output_type = ZOPFLI_FORMAT_GZIP;
    else if (StringsEqual(arg, "--splitlast"))  ;
    else if (arg[0] == '-' && arg[1] == '-' && arg[2] == 'i'
        && arg[3] >= '0' && arg[3] <= '9') {
      options.numiterations = atoi(arg + 3);
    }
    else if (StringsEqual(arg, "-h")) {
      fprintf(stderr,
          "Usage: zopfli [OPTION]... FILE...\n"
          "  -h    gives this help\n"
          "  -c    write the result on standard output, instead of disk"
          " filename + '.gz'\n"
          "  -v    verbose mode\n"
          "  --i#  perform # iterations (default 15). More gives"
          " more compression but is slower."
          " Examples: --i10, --i50, --i1000\n");
      fprintf(stderr,
          "  --gzip        output to gzip format (default)\n"
          "  --zlib        output to zlib format instead of gzip\n"
          "  --deflate     output to deflate format instead of gzip\n"
          "  --splitlast   ignored, left for backwards compatibility\n");
      return 0;
    }
  }
  if (options.numiterations < 1) {
    fprintf(stderr, "Error: must have 1 or more iterations\n");
    return 0;
  }
  for (i = 1; i < argc; i++) {
    if (argv[i][0] != '-') {
      char* outfilename;
      filename = argv[i];
      if (output_to_stdout) {
        outfilename = 0;
      } else if (output_type == ZOPFLI_FORMAT_GZIP) {
        outfilename = AddStrings(filename, ".gz");
      } else if (output_type == ZOPFLI_FORMAT_ZLIB) {
        outfilename = AddStrings(filename, ".zlib");
      } else {
        assert(output_type == ZOPFLI_FORMAT_DEFLATE);
        outfilename = AddStrings(filename, ".deflate");
      }
      if (options.verbose && outfilename) {
        fprintf(stderr, "Saving to: %s\n", outfilename);
      }
      CompressFile(&options, output_type, filename, outfilename);
      free(outfilename);
    }
  }
  if (!filename) {
    fprintf(stderr,
            "Please provide filename\nFor help, type: %s -h\n", argv[0]);
  }
  return 0;
}
```
