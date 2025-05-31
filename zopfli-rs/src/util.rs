// Core constants from src/zopfli/util.h
pub const ZOPFLI_MAX_MATCH: usize = 258;
pub const ZOPFLI_MIN_MATCH: usize = 3;
pub const ZOPFLI_NUM_LL: usize = 288;
pub const ZOPFLI_NUM_D: usize = 32;
pub const ZOPFLI_WINDOW_SIZE: usize = 32768;
pub const ZOPFLI_WINDOW_MASK: usize = ZOPFLI_WINDOW_SIZE - 1;
pub const ZOPFLI_MASTER_BLOCK_SIZE: usize = 1000000;
pub const ZOPFLI_LARGE_FLOAT: f64 = 1e30;
pub const ZOPFLI_CACHE_LENGTH: usize = 8;
pub const ZOPFLI_MAX_CHAIN_HITS: usize = 8192;