use crate::options::ZopfliOptions;
use crate::gzip_container::gzip_compress;
use crate::zlib_container::zlib_compress;
use crate::deflate::{deflate, BitWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZopfliFormat {
    Gzip = 0,
    Zlib = 1,
    Deflate = 2,
}

impl From<i32> for ZopfliFormat {
    fn from(value: i32) -> Self {
        match value {
            0 => ZopfliFormat::Gzip,
            1 => ZopfliFormat::Zlib,
            2 => ZopfliFormat::Deflate,
            _ => panic!("Invalid ZopfliFormat value: {}", value),
        }
    }
}

impl From<ZopfliFormat> for i32 {
    fn from(format: ZopfliFormat) -> Self {
        format as i32
    }
}

pub fn compress(
    options: &ZopfliOptions,
    output_type: ZopfliFormat,
    input: &[u8],
    output: &mut Vec<u8>
) {
    match output_type {
        ZopfliFormat::Gzip => {
            gzip_compress(options, input, output);
        }
        ZopfliFormat::Zlib => {
            zlib_compress(options, input, output);
        }
        ZopfliFormat::Deflate => {
            let mut writer = BitWriter::new();
            deflate(options, 2, true, input, input.len(), &mut writer);
            output.extend_from_slice(writer.get_data());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zopfli_format_conversion() {
        assert_eq!(ZopfliFormat::from(0), ZopfliFormat::Gzip);
        assert_eq!(ZopfliFormat::from(1), ZopfliFormat::Zlib);
        assert_eq!(ZopfliFormat::from(2), ZopfliFormat::Deflate);
        
        assert_eq!(i32::from(ZopfliFormat::Gzip), 0);
        assert_eq!(i32::from(ZopfliFormat::Zlib), 1);
        assert_eq!(i32::from(ZopfliFormat::Deflate), 2);
    }

    #[cfg(feature = "c-fallback")]
    #[test]
    fn test_compress_against_c() {
        use std::ptr;
        use std::os::raw::c_uchar;

        let test_data = b"Hello, World! This is a test string for compression.";
        let options = ZopfliOptions::default();
        
        for &format in &[ZopfliFormat::Gzip, ZopfliFormat::Zlib, ZopfliFormat::Deflate] {
            let mut rust_output = Vec::new();
            compress(&options, format, test_data, &mut rust_output);
            
            // Call C compress
            let mut c_output_ptr: *mut c_uchar = ptr::null_mut();
            let mut c_output_size = 0usize;
            
            unsafe {
                crate::ffi::ZopfliCompress(
                    &options as *const _,
                    format as i32,
                    test_data.as_ptr(),
                    test_data.len(),
                    &mut c_output_ptr,
                    &mut c_output_size
                );
                
                let c_output = std::slice::from_raw_parts(c_output_ptr, c_output_size).to_vec();
                libc::free(c_output_ptr as *mut libc::c_void);
                
                assert_eq!(rust_output, c_output, "Outputs don't match for format {:?}", format);
            }
        }
    }

    #[test]
    fn test_compress_basic() {
        let options = ZopfliOptions::default();
        let input = b"Hello, World!";
        
        for &format in &[ZopfliFormat::Gzip, ZopfliFormat::Zlib, ZopfliFormat::Deflate] {
            let mut output = Vec::new();
            compress(&options, format, input, &mut output);
            
            assert!(!output.is_empty(), "Output should not be empty for format {:?}", format);
            
            match format {
                ZopfliFormat::Gzip => {
                    assert_eq!(output[0], 31, "Gzip magic number 1");
                    assert_eq!(output[1], 139, "Gzip magic number 2");
                }
                ZopfliFormat::Zlib => {
                    // Check valid zlib header
                    let cmf = output[0] as u32;
                    let flg = output[1] as u32;
                    assert_eq!((cmf * 256 + flg) % 31, 0, "Invalid zlib header");
                }
                ZopfliFormat::Deflate => {
                    // Deflate format has no specific header to check
                    assert!(output.len() > 0, "Deflate output should not be empty");
                }
            }
        }
    }
}