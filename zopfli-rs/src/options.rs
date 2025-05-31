use std::os::raw::c_int;

/// Options used throughout the Zopfli compression program.
/// This struct mirrors the C ZopfliOptions structure exactly.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZopfliOptions {
    /// Whether to print output
    pub verbose: c_int,

    /// Whether to print more detailed output
    pub verbose_more: c_int,

    /// Maximum amount of times to rerun forward and backward pass to optimize LZ77
    /// compression cost. Good values: 10, 15 for small files, 5 for files over
    /// several MB in size or it will be too slow.
    pub numiterations: c_int,

    /// If true, splits the data in multiple deflate blocks with optimal choice
    /// for the block boundaries. Block splitting gives better compression. Default:
    /// true (1).
    pub blocksplitting: c_int,

    /// No longer used, left for compatibility.
    pub blocksplittinglast: c_int,

    /// Maximum amount of blocks to split into (0 for unlimited, but this can give
    /// extreme results that hurt compression on some files). Default value: 15.
    pub blocksplittingmax: c_int,
}

impl Default for ZopfliOptions {
    /// Initializes options with default values.
    /// Equivalent to C function ZopfliInitOptions.
    fn default() -> Self {
        ZopfliOptions {
            verbose: 0,
            verbose_more: 0,
            numiterations: 15,
            blocksplitting: 1,
            blocksplittinglast: 0,
            blocksplittingmax: 15,
        }
    }
}

impl ZopfliOptions {
    /// Creates a new ZopfliOptions with default values.
    /// Equivalent to calling ZopfliInitOptions in C.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let options = ZopfliOptions::default();
        assert_eq!(options.verbose, 0);
        assert_eq!(options.verbose_more, 0);
        assert_eq!(options.numiterations, 15);
        assert_eq!(options.blocksplitting, 1);
        assert_eq!(options.blocksplittinglast, 0);
        assert_eq!(options.blocksplittingmax, 15);
    }

    #[test]
    fn test_new_equals_default() {
        let options1 = ZopfliOptions::new();
        let options2 = ZopfliOptions::default();
        assert_eq!(options1, options2);
    }

    #[test]
    fn test_struct_size() {
        // Verify that our Rust struct has the same size as the C struct
        // This is critical for FFI compatibility
        assert_eq!(
            std::mem::size_of::<ZopfliOptions>(),
            6 * std::mem::size_of::<c_int>()
        );
    }
}