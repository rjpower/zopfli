#include "symbols.h"

// Wrapper function for static inline function that can't be linked directly
int zopfli_get_dist_extra_bits_wrapper(int dist) {
    return ZopfliGetDistExtraBits(dist);
}

// Wrapper for ZopfliGetDistExtraBitsValue
int zopfli_get_dist_extra_bits_value_wrapper(int dist) {
    return ZopfliGetDistExtraBitsValue(dist);
}

// Wrapper for ZopfliGetDistSymbol
int zopfli_get_dist_symbol_wrapper(int dist) {
    return ZopfliGetDistSymbol(dist);
}

// Wrapper for ZopfliGetLengthExtraBits
int zopfli_get_length_extra_bits_wrapper(int l) {
    return ZopfliGetLengthExtraBits(l);
}

// Wrapper for ZopfliGetLengthExtraBitsValue
int zopfli_get_length_extra_bits_value_wrapper(int l) {
    return ZopfliGetLengthExtraBitsValue(l);
}

// Wrapper for ZopfliGetLengthSymbol
int zopfli_get_length_symbol_wrapper(int l) {
    return ZopfliGetLengthSymbol(l);
}

// Wrapper for ZopfliGetLengthSymbolExtraBits
int zopfli_get_length_symbol_extra_bits_wrapper(int s) {
    return ZopfliGetLengthSymbolExtraBits(s);
}

// Wrapper for ZopfliGetDistSymbolExtraBits
int zopfli_get_dist_symbol_extra_bits_wrapper(int s) {
    return ZopfliGetDistSymbolExtraBits(s);
}