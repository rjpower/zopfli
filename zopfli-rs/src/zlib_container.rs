use crate::options::ZopfliOptions;
use crate::deflate::{deflate, BitWriter};

pub fn adler32(data: &[u8]) -> u32 {
    const SUMS_OVERFLOW: usize = 5550;
    let mut s1 = 1u32;
    let mut s2 = 1u32 >> 16;  // This is 0 since 1 >> 16 = 0

    let mut remaining = data.len();
    let mut pos = 0;

    while remaining > 0 {
        let amount = if remaining > SUMS_OVERFLOW {
            SUMS_OVERFLOW
        } else {
            remaining
        };
        remaining -= amount;

        for _ in 0..amount {
            s1 += data[pos] as u32;
            s2 += s1;
            pos += 1;
        }
        
        s1 %= 65521;
        s2 %= 65521;
    }

    (s2 << 16) | s1
}

pub fn zlib_compress(
    options: &ZopfliOptions,
    input: &[u8],
    output: &mut Vec<u8>
) {
    let checksum = adler32(input);
    let cmf = 120u32;  // CM 8, CINFO 7. See zlib spec.
    let flevel = 3u32;
    let fdict = 0u32;
    let mut cmfflg = 256 * cmf + fdict * 32 + flevel * 64;
    let fcheck = 31 - cmfflg % 31;
    cmfflg += fcheck;

    // Zlib header (2 bytes)
    output.push((cmfflg / 256) as u8);
    output.push((cmfflg % 256) as u8);

    // Compress the actual data using DEFLATE
    let mut writer = BitWriter::new();
    deflate(options, 2, true, input, input.len(), &mut writer);
    output.extend_from_slice(writer.get_data());

    // Adler-32 checksum (big-endian, 4 bytes)
    output.push(((checksum >> 24) & 0xff) as u8);
    output.push(((checksum >> 16) & 0xff) as u8);
    output.push(((checksum >> 8) & 0xff) as u8);
    output.push((checksum & 0xff) as u8);

    if options.verbose != 0 {
        eprintln!(
            "Original Size: {}, Zlib: {}, Compression: {:.1}% Removed",
            input.len(),
            output.len(),
            100.0 * (input.len() as f64 - output.len() as f64) / input.len() as f64
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32_basic() {
        // Test with known values
        assert_eq!(adler32(b""), 1);
        assert_eq!(adler32(b"a"), 0x00620062);
        assert_eq!(adler32(b"abc"), 0x024d0127);
        assert_eq!(adler32(b"message digest"), 0x29750586);
        assert_eq!(adler32(b"abcdefghijklmnopqrstuvwxyz"), 0x90860b20);
    }

    #[cfg(feature = "c-fallback")]
    #[test]
    fn test_adler32_against_c() {
        use std::ptr;
        use std::os::raw::c_uchar;

        let test_data = b"Hello, World! This is a test string for Adler-32 calculation.";
        
        // Calculate Adler-32 with Rust implementation
        let _rust_adler = adler32(test_data);
        
        // Since C doesn't expose adler32 function directly, we test by comparing
        // the full zlib output which includes the Adler-32
        let options = ZopfliOptions::default();
        let mut rust_output = Vec::new();
        zlib_compress(&options, test_data, &mut rust_output);
        
        // Call C zlib compress
        let mut c_output_ptr: *mut c_uchar = ptr::null_mut();
        let mut c_output_size = 0usize;
        
        unsafe {
            crate::ffi::ZopfliZlibCompress(
                &options as *const _,
                test_data.as_ptr(),
                test_data.len(),
                &mut c_output_ptr,
                &mut c_output_size
            );
            
            let c_output = std::slice::from_raw_parts(c_output_ptr, c_output_size).to_vec();
            libc::free(c_output_ptr as *mut libc::c_void);
            
            // Compare the entire outputs
            assert_eq!(rust_output, c_output, "Zlib outputs don't match");
        }
    }

    #[test]
    fn test_zlib_basic() {
        let options = ZopfliOptions::default();
        let input = b"Hello, World!";
        let mut output = Vec::new();
        
        zlib_compress(&options, input, &mut output);
        
        // Basic sanity checks
        assert!(output.len() > 6, "Output should be longer than zlib header + checksum");
        // Check that we have a valid zlib header
        let cmf = output[0] as u32;
        let flg = output[1] as u32;
        assert_eq!((cmf * 256 + flg) % 31, 0, "Invalid zlib header checksum");
    }
}