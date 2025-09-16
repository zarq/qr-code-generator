use crate::types::Version;

pub fn get_alignment_positions(version: Version) -> Vec<usize> {
    match version {
        Version::V1 => vec![],
        Version::V2 => vec![6, 18],
        Version::V3 => vec![6, 22],
        Version::V4 => vec![6, 26],
        Version::V5 => vec![6, 30],
        Version::V6 => vec![6, 34],
        Version::V7 => vec![6, 22, 38],
        Version::V8 => vec![6, 24, 42],
        Version::V9 => vec![6, 26, 46],
        Version::V10 => vec![6, 28, 50],
        Version::V11 => vec![6, 30, 54],
        Version::V12 => vec![6, 32, 58],
        Version::V13 => vec![6, 26, 46, 66],
        Version::V14 => vec![6, 26, 46, 66],
        Version::V15 => vec![6, 26, 48, 70],
        Version::V16 => vec![6, 26, 50, 74],
        Version::V17 => vec![6, 30, 54, 78],
        Version::V18 => vec![6, 30, 56, 82],
        Version::V19 => vec![6, 30, 58, 86],
        Version::V20 => vec![6, 34, 62, 90],
        Version::V21 => vec![6, 28, 50, 72, 94],
        Version::V22 => vec![6, 26, 50, 74, 98],
        Version::V23 => vec![6, 30, 54, 78, 102],
        Version::V24 => vec![6, 28, 54, 80, 106],
        Version::V25 => vec![6, 32, 58, 84, 110],
        Version::V26 => vec![6, 30, 58, 86, 114],
        Version::V27 => vec![6, 34, 62, 90, 118],
        Version::V28 => vec![6, 26, 50, 74, 98, 122],
        Version::V29 => vec![6, 30, 54, 78, 102, 126],
        Version::V30 => vec![6, 26, 52, 78, 104, 130],
        Version::V31 => vec![6, 30, 56, 82, 108, 134],
        Version::V32 => vec![6, 34, 60, 86, 112, 138],
        Version::V33 => vec![6, 30, 58, 86, 114, 142],
        Version::V34 => vec![6, 34, 62, 90, 118, 146],
        Version::V35 => vec![6, 30, 54, 78, 102, 126, 150],
        Version::V36 => vec![6, 24, 50, 76, 102, 128, 154],
        Version::V37 => vec![6, 28, 54, 80, 106, 132, 158],
        Version::V38 => vec![6, 32, 58, 84, 110, 136, 162],
        Version::V39 => vec![6, 26, 54, 82, 110, 138, 166],
        Version::V40 => vec![6, 30, 58, 86, 114, 142, 170],
    }
}

pub fn is_alignment_pattern(x: usize, y: usize, version: Version) -> bool {
    let positions = get_alignment_positions(version);
    if positions.is_empty() {
        return false;
    }
    
    for &center_x in &positions {
        for &center_y in &positions {
            // Skip if overlaps with finder patterns
            if (center_x <= 8 && center_y <= 8) ||
               (center_x <= 8 && center_y >= version.size() - 9) ||
               (center_x >= version.size() - 9 && center_y <= 8) {
                continue;
            }
            
            if x >= center_x.saturating_sub(2) && x <= center_x + 2 &&
               y >= center_y.saturating_sub(2) && y <= center_y + 2 {
                return true;
            }
        }
    }
    false
}
