//! Precomputed gamma tables (functions) for fixed values.
//!
//! Precomputes f(x) = ((x/256) ^ gamma ) * 256
//!
//! Where the domain x and range f(x) are integers, gamma is a constant.  The tables are used
//! to quickly do color adjustments on 8 bit images.
//!

// Table for f(x) = ((x/256) ^ 1.7 ) * 256
//
pub const GAMMA_17: [u16; 257] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7,
    7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 17, 17, 18, 18, 19,
    19, 20, 21, 21, 22, 22, 23, 24, 24, 25, 26, 26, 27, 28, 28, 29, 30, 31, 31, 32, 33, 33, 34, 35,
    36, 36, 37, 38, 39, 40, 40, 41, 42, 43, 44, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 95, 96, 97, 98, 99, 100, 102, 103, 104,
    105, 106, 107, 109, 110, 111, 112, 113, 115, 116, 117, 118, 120, 121, 122, 123, 125, 126, 127,
    128, 130, 131, 132, 134, 135, 136, 138, 139, 140, 141, 143, 144, 146, 147, 148, 150, 151, 152,
    154, 155, 156, 158, 159, 161, 162, 163, 165, 166, 168, 169, 171, 172, 174, 175, 176, 178, 179,
    181, 182, 184, 185, 187, 188, 190, 191, 193, 194, 196, 197, 199, 200, 202, 204, 205, 207, 208,
    210, 211, 213, 214, 216, 218, 219, 221, 222, 224, 226, 227, 229, 231, 232, 234, 235, 237, 239,
    240, 242, 244, 245, 247, 249, 250, 252, 254, 256,
];

// Table for f(x) = ((x/256) ^ 3.0 ) * 256
//
pub const GAMMA_30: [u16; 257] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
    12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 22, 22, 23, 23,
    24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 40, 40, 41,
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 63, 64, 66, 67,
    68, 69, 71, 72, 73, 74, 76, 77, 79, 80, 81, 83, 84, 86, 87, 88, 90, 91, 93, 95, 96, 98, 99,
    101, 103, 104, 106, 108, 109, 111, 113, 114, 116, 118, 120, 122, 123, 125, 127, 129, 131, 133,
    135, 137, 139, 141, 143, 145, 147, 149, 151, 153, 155, 158, 160, 162, 164, 166, 169, 171, 173,
    176, 178, 180, 183, 185, 188, 190, 193, 195, 198, 200, 203, 205, 208, 210, 213, 216, 218, 221,
    224, 227, 229, 232, 235, 238, 241, 244, 247, 250, 253, 256,
];

// Table for f(x) = ((x/256) ^ 2.2 ) * 256
//
pub const GAMMA_22: [u16; 257] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
    2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10,
    11, 11, 12, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 22, 22,
    23, 23, 24, 25, 25, 26, 26, 27, 28, 28, 29, 30, 30, 31, 32, 33, 33, 34, 35, 36, 36, 37, 38, 39,
    39, 40, 41, 42, 43, 44, 44, 45, 46, 47, 48, 49, 50, 51, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
    61, 62, 63, 64, 65, 66, 67, 68, 70, 71, 72, 73, 74, 75, 76, 77, 78, 80, 81, 82, 83, 84, 86, 87,
    88, 89, 91, 92, 93, 94, 96, 97, 98, 100, 101, 102, 104, 105, 106, 108, 109, 110, 112, 113, 115,
    116, 117, 119, 120, 122, 123, 125, 126, 128, 129, 131, 132, 134, 135, 137, 139, 140, 142, 143,
    145, 147, 148, 150, 152, 153, 155, 157, 158, 160, 162, 163, 165, 167, 169, 170, 172, 174, 176,
    177, 179, 181, 183, 185, 187, 188, 190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212,
    214, 216, 218, 220, 222, 224, 226, 228, 230, 232, 234, 236, 238, 240, 242, 245, 247, 249, 251,
    253, 256,
];

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function that compute tables algorithmicly.
    fn compute_expected(gamma: f32) -> Vec<u16> {
        let mut outvec = Vec::new();
        for n in 0..=256 {
            let input = (n as f32) / 256.0;
            let output = input.powf(gamma);
            outvec.push((output * 256.0) as u16);
        }
        outvec
    }

    #[test]
    fn should_match_computed_output() {
        assert_eq!(compute_expected(1.7), GAMMA_17);
        assert_eq!(compute_expected(2.2), GAMMA_22);
        assert_eq!(compute_expected(3.0), GAMMA_30);
    }
}
