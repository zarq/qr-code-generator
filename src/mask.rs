use crate::types::MaskPattern;

pub fn apply_mask(matrix: &mut Vec<Vec<u8>>, pattern: MaskPattern) {
    match pattern {
        MaskPattern::Pattern0 => apply_pattern0(matrix),
        MaskPattern::Pattern1 => apply_pattern1(matrix),
        MaskPattern::Pattern2 => apply_pattern2(matrix),
        MaskPattern::Pattern3 => apply_pattern3(matrix),
        MaskPattern::Pattern4 => apply_pattern4(matrix),
        MaskPattern::Pattern5 => apply_pattern5(matrix),
        MaskPattern::Pattern6 => apply_pattern6(matrix),
        MaskPattern::Pattern7 => apply_pattern7(matrix),
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

fn apply_pattern2(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if x % 3 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern3(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if (x + y) % 3 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern4(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if ((y / 2) + (x / 3)) % 2 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern5(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if ((x * y) % 2) + ((x * y) % 3) == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern6(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if (((x * y) % 2) + ((x * y) % 3)) % 2 == 0 {
                matrix[y][x] ^= 1;
            }
        }
    }
}

fn apply_pattern7(matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            if (((x + y) % 2) + ((x * y) % 3)) % 2 == 0 {
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
