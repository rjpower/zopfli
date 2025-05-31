/// Gets the amount of extra bits for the given dist, cfr. the DEFLATE spec.
pub fn get_dist_extra_bits(dist: i32) -> i32 {
    if dist < 5 {
        return 0;
    }
    
    // Using leading_zeros() which is equivalent to __builtin_clz
    let l = 31 - (dist - 1).leading_zeros() as i32;
    l - 1 // log2(dist - 1) - 1
}

/// Gets value of the extra bits for the given dist, cfr. the DEFLATE spec.
pub fn get_dist_extra_bits_value(dist: i32) -> i32 {
    if dist < 5 {
        return 0;
    }
    
    // Using leading_zeros() which is equivalent to __builtin_clz
    let l = 31 - (dist - 1).leading_zeros() as i32; // log2(dist - 1)
    (dist - (1 + (1 << l))) & ((1 << (l - 1)) - 1)
}

/// Gets the symbol for the given dist, cfr. the DEFLATE spec.
pub fn get_dist_symbol(dist: i32) -> i32 {
    if dist < 5 {
        return dist - 1;
    }
    
    // Using leading_zeros() which is equivalent to __builtin_clz
    let l = 31 - (dist - 1).leading_zeros() as i32; // log2(dist - 1)
    let r = ((dist - 1) >> (l - 1)) & 1;
    l * 2 + r
}

/// Gets the amount of extra bits for the given length, cfr. the DEFLATE spec.
pub fn get_length_extra_bits(l: i32) -> i32 {
    if l < 0 || l > 258 {
        return 0; // Out of bounds
    }
    
    // Static table from C implementation
    const TABLE: [i32; 259] = [
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
    ];
    TABLE[l as usize]
}

/// Gets value of the extra bits for the given length, cfr. the DEFLATE spec.
pub fn get_length_extra_bits_value(l: i32) -> i32 {
    if l < 0 || l > 258 {
        return 0; // Out of bounds
    }
    
    // Static table from C implementation
    const TABLE: [i32; 259] = [
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
    ];
    TABLE[l as usize]
}

/// Gets the symbol for the given length, cfr. the DEFLATE spec.
/// Returns the symbol in the range [257-285] (inclusive)
pub fn get_length_symbol(l: i32) -> i32 {
    if l < 0 || l > 258 {
        return 0; // Out of bounds
    }
    
    // Static table from C implementation
    const TABLE: [i32; 259] = [
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
    ];
    TABLE[l as usize]
}

/// Gets the amount of extra bits for the given length symbol.
pub fn get_length_symbol_extra_bits(s: i32) -> i32 {
    if s < 257 || s > 285 {
        return 0; // Out of bounds
    }
    
    // Static table from C implementation
    const TABLE: [i32; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
        3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
    ];
    TABLE[(s - 257) as usize]
}

/// Gets the amount of extra bits for the given distance symbol.
pub fn get_dist_symbol_extra_bits(s: i32) -> i32 {
    if s < 0 || s > 29 {
        return 0; // Out of bounds
    }
    
    // Static table from C implementation
    const TABLE: [i32; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
        9, 9, 10, 10, 11, 11, 12, 12, 13, 13
    ];
    TABLE[s as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dist_extra_bits() {
        // Test edge cases and known values
        assert_eq!(get_dist_extra_bits(1), 0);
        assert_eq!(get_dist_extra_bits(2), 0);
        assert_eq!(get_dist_extra_bits(3), 0);
        assert_eq!(get_dist_extra_bits(4), 0);
        assert_eq!(get_dist_extra_bits(5), 1);
        assert_eq!(get_dist_extra_bits(8), 1);
        assert_eq!(get_dist_extra_bits(9), 2);
        assert_eq!(get_dist_extra_bits(16), 2);
        assert_eq!(get_dist_extra_bits(17), 3);
        assert_eq!(get_dist_extra_bits(32), 3);
        assert_eq!(get_dist_extra_bits(33), 4);
        assert_eq!(get_dist_extra_bits(64), 4);
        assert_eq!(get_dist_extra_bits(65), 5);
        assert_eq!(get_dist_extra_bits(128), 5);
        assert_eq!(get_dist_extra_bits(129), 6);
        assert_eq!(get_dist_extra_bits(256), 6);
        assert_eq!(get_dist_extra_bits(257), 7);
        assert_eq!(get_dist_extra_bits(512), 7);
        assert_eq!(get_dist_extra_bits(513), 8);
        assert_eq!(get_dist_extra_bits(1024), 8);
        assert_eq!(get_dist_extra_bits(1025), 9);
        assert_eq!(get_dist_extra_bits(2048), 9);
        assert_eq!(get_dist_extra_bits(2049), 10);
        assert_eq!(get_dist_extra_bits(4096), 10);
        assert_eq!(get_dist_extra_bits(4097), 11);
        assert_eq!(get_dist_extra_bits(8192), 11);
        assert_eq!(get_dist_extra_bits(8193), 12);
        assert_eq!(get_dist_extra_bits(16384), 12);
        assert_eq!(get_dist_extra_bits(16385), 13);
        assert_eq!(get_dist_extra_bits(32768), 13);
    }

    #[test]
    fn test_get_dist_extra_bits_value() {
        // Test edge cases and known values
        assert_eq!(get_dist_extra_bits_value(1), 0);
        assert_eq!(get_dist_extra_bits_value(2), 0);
        assert_eq!(get_dist_extra_bits_value(3), 0);
        assert_eq!(get_dist_extra_bits_value(4), 0);
        assert_eq!(get_dist_extra_bits_value(5), 0); // (5 - 5) & 1 = 0
        assert_eq!(get_dist_extra_bits_value(6), 1); // (6 - 5) & 1 = 1
        assert_eq!(get_dist_extra_bits_value(7), 0); // (7 - 5) & 1 = 0
        assert_eq!(get_dist_extra_bits_value(8), 1); // (8 - 5) & 1 = 1
        assert_eq!(get_dist_extra_bits_value(9), 0); // (9 - 9) & 3 = 0
        assert_eq!(get_dist_extra_bits_value(10), 1); // (10 - 9) & 3 = 1
        assert_eq!(get_dist_extra_bits_value(11), 2); // (11 - 9) & 3 = 2
        assert_eq!(get_dist_extra_bits_value(12), 3); // (12 - 9) & 3 = 3
        assert_eq!(get_dist_extra_bits_value(16), 3); // (16 - 9) & 3 = 3
        assert_eq!(get_dist_extra_bits_value(17), 0); // (17 - 17) & 7 = 0
    }
}