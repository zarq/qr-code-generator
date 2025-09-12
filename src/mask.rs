use crate::types::MaskPattern;

pub fn apply_mask(matrix: &mut Vec<Vec<u8>>, pattern: MaskPattern) {
    match pattern {
        MaskPattern::Pattern0 => apply_pattern0(matrix),
        MaskPattern::Pattern1 => apply_pattern1(matrix),
        _ => {} // Other patterns not implemented yet
    }
}

fn apply_pattern0(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if (x + y) % 2 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern1(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if y % 2 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

impl Default for MaskPattern {
    fn default() -> Self {
        MaskPattern::Pattern0
    }
}
