use crate::options::ZopfliOptions;
use crate::deflate::{deflate, BitWriter};

static CRC32_TABLE: [u32; 256] = [
           0, 1996959894, 3993919788, 2567524794,  124634137, 1886057615,
  3915621685, 2657392035,  249268274, 2044508324, 3772115230, 2547177864,
   162941995, 2125561021, 3887607047, 2428444049,  498536548, 1789927666,
  4089016648, 2227061214,  450548861, 1843258603, 4107580753, 2211677639,
   325883990, 1684777152, 4251122042, 2321926636,  335633487, 1661365465,
  4195302755, 2366115317,  997073096, 1281953886, 3579855332, 2724688242,
  1006888145, 1258607687, 3524101629, 2768942443,  901097722, 1119000684,
  3686517206, 2898065728,  853044451, 1172266101, 3705015759, 2882616665,
   651767980, 1373503546, 3369554304, 3218104598,  565507253, 1454621731,
  3485111705, 3099436303,  671266974, 1594198024, 3322730930, 2970347812,
   795835527, 1483230225, 3244367275, 3060149565, 1994146192,   31158534,
  2563907772, 4023717930, 1907459465,  112637215, 2680153253, 3904427059,
  2013776290,  251722036, 2517215374, 3775830040, 2137656763,  141376813,
  2439277719, 3865271297, 1802195444,  476864866, 2238001368, 4066508878,
  1812370925,  453092731, 2181625025, 4111451223, 1706088902,  314042704,
  2344532202, 4240017532, 1658658271,  366619977, 2362670323, 4224994405,
  1303535960,  984961486, 2747007092, 3569037538, 1256170817, 1037604311,
  2765210733, 3554079995, 1131014506,  879679996, 2909243462, 3663771856,
  1141124467,  855842277, 2852801631, 3708648649, 1342533948,  654459306,
  3188396048, 3373015174, 1466479909,  544179635, 3110523913, 3462522015,
  1591671054,  702138776, 2966460450, 3352799412, 1504918807,  783551873,
  3082640443, 3233442989, 3988292384, 2596254646,   62317068, 1957810842,
  3939845945, 2647816111,   81470997, 1943803523, 3814918930, 2489596804,
   225274430, 2053790376, 3826175755, 2466906013,  167816743, 2097651377,
  4027552580, 2265490386,  503444072, 1762050814, 4150417245, 2154129355,
   426522225, 1852507879, 4275313526, 2312317920,  282753626, 1742555852,
  4189708143, 2394877945,  397917763, 1622183637, 3604390888, 2714866558,
   953729732, 1340076626, 3518719985, 2797360999, 1068828381, 1219638859,
  3624741850, 2936675148,  906185462, 1090812512, 3747672003, 2825379669,
   829329135, 1181335161, 3412177804, 3160834842,  628085408, 1382605366,
  3423369109, 3138078467,  570562233, 1426400815, 3317316542, 2998733608,
   733239954, 1555261956, 3268935591, 3050360625,  752459403, 1541320221,
  2607071920, 3965973030, 1969922972,   40735498, 2617837225, 3943577151,
  1913087877,   83908371, 2512341634, 3803740692, 2075208622,  213261112,
  2463272603, 3855990285, 2094854071,  198958881, 2262029012, 4057260610,
  1759359992,  534414190, 2176718541, 4139329115, 1873836001,  414664567,
  2282248934, 4279200368, 1711684554,  285281116, 2405801727, 4167216745,
  1634467795,  376229701, 2685067896, 3608007406, 1308918612,  956543938,
  2808555105, 3495958263, 1231636301, 1047427035, 2932959818, 3654703836,
  1088359270,  936918000, 2847714899, 3736837829, 1202900863,  817233897,
  3183342108, 3401237130, 1404277552,  615818150, 3134207493, 3453421203,
  1423857449,  601450431, 3009837614, 3294710456, 1567103746,  711928724,
  3020668471, 3272380065, 1510334235,  755167117
];

pub fn crc32(data: &[u8]) -> u32 {
    let mut result = 0xffffffffu32;
    for &byte in data {
        result = CRC32_TABLE[((result ^ (byte as u32)) & 0xff) as usize] ^ (result >> 8);
    }
    result ^ 0xffffffffu32
}

pub fn gzip_compress(
    options: &ZopfliOptions,
    input: &[u8],
    output: &mut Vec<u8>
) {
    let crcvalue = crc32(input);

    // Gzip header
    output.push(31);    // ID1
    output.push(139);   // ID2
    output.push(8);     // CM (Compression Method: DEFLATE)
    output.push(0);     // FLG (Flags)
    
    // MTIME (4 bytes, set to 0)
    output.push(0);
    output.push(0);
    output.push(0);
    output.push(0);
    
    output.push(2);     // XFL (Extra flags: 2 indicates best compression)
    output.push(3);     // OS (Operating System: 3 follows Unix conventions)

    // Compress the actual data using DEFLATE
    let mut writer = BitWriter::new();
    deflate(options, 2, true, input, input.len(), &mut writer);
    output.extend_from_slice(writer.get_data());

    // CRC32 (little-endian)
    output.push((crcvalue & 0xff) as u8);
    output.push(((crcvalue >> 8) & 0xff) as u8);
    output.push(((crcvalue >> 16) & 0xff) as u8);
    output.push(((crcvalue >> 24) & 0xff) as u8);

    // ISIZE - input size (little-endian)
    let insize = input.len() as u32;
    output.push((insize & 0xff) as u8);
    output.push(((insize >> 8) & 0xff) as u8);
    output.push(((insize >> 16) & 0xff) as u8);
    output.push(((insize >> 24) & 0xff) as u8);

    if options.verbose != 0 {
        eprintln!(
            "Original Size: {}, Gzip: {}, Compression: {:.1}% Removed",
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
    fn test_crc32_basic() {
        // Test with known values
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"a"), 0xe8b7be43);
        assert_eq!(crc32(b"abc"), 0x352441c2);
        assert_eq!(crc32(b"message digest"), 0x20159d7f);
        assert_eq!(crc32(b"abcdefghijklmnopqrstuvwxyz"), 0x4c2750bd);
    }

    #[cfg(feature = "c-fallback")]
    #[test]
    fn test_crc32_against_c() {
        use std::ptr;
        use std::os::raw::c_uchar;

        let test_data = b"Hello, World! This is a test string for CRC32 calculation.";
        
        // Calculate CRC with Rust implementation
        let _rust_crc = crc32(test_data);
        
        // Since C doesn't expose CRC function directly, we test by comparing
        // the full gzip output which includes the CRC
        let options = ZopfliOptions::default();
        let mut rust_output = Vec::new();
        gzip_compress(&options, test_data, &mut rust_output);
        
        // Call C gzip compress
        let mut c_output_ptr: *mut c_uchar = ptr::null_mut();
        let mut c_output_size = 0usize;
        
        unsafe {
            crate::ffi::ZopfliGzipCompress(
                &options as *const _,
                test_data.as_ptr(),
                test_data.len(),
                &mut c_output_ptr,
                &mut c_output_size
            );
            
            let c_output = std::slice::from_raw_parts(c_output_ptr, c_output_size).to_vec();
            libc::free(c_output_ptr as *mut libc::c_void);
            
            // Compare the entire outputs
            assert_eq!(rust_output, c_output, "Gzip outputs don't match");
        }
    }

    #[test]
    fn test_gzip_basic() {
        let options = ZopfliOptions::default();
        let input = b"Hello, World!";
        let mut output = Vec::new();
        
        gzip_compress(&options, input, &mut output);
        
        // Basic sanity checks
        assert!(output.len() > 10, "Output should be longer than gzip header");
        assert_eq!(output[0], 31, "First byte should be gzip magic number 31");
        assert_eq!(output[1], 139, "Second byte should be gzip magic number 139");
        assert_eq!(output[2], 8, "Third byte should be compression method DEFLATE");
    }
}