#include "../../src/zopfli/lz77.h"

// Helper functions to access C LZ77Store internals for testing
size_t ZopfliLZ77StoreGetSize(const ZopfliLZ77Store* store) {
    return store->size;
}

unsigned short ZopfliLZ77StoreGetLitLen(const ZopfliLZ77Store* store, size_t index) {
    return store->litlens[index];
}

unsigned short ZopfliLZ77StoreGetDist(const ZopfliLZ77Store* store, size_t index) {
    return store->dists[index];
}

size_t ZopfliLZ77StoreGetPos(const ZopfliLZ77Store* store, size_t index) {
    return store->pos[index];
}