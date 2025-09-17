use crate::types::Version;
use crate::alignment::get_alignment_positions;

/// Get all data and ECC pixel positions for a given QR code version
pub fn get_data_ecc_positions(version: Version) -> Vec<(usize, usize)> {
    let size = version_to_size(version);
    let mut positions = Vec::new();
    
    // Read data in zigzag pattern (right to left, alternating up/down)
    let mut col = size - 1;
    let mut going_up = true;
    
    while col > 0 {
        // Skip timing column
        if col == 6 {
            col -= 1;
            continue;
        }
        
        // Process two columns at a time
        for c in [col, col - 1] {
            let rows: Vec<usize> = if going_up {
                (0..size).rev().collect()
            } else {
                (0..size).collect()
            };
            
            for row in rows {
                if !is_function_module(row, c, size) {
                    positions.push((row, c));
                }
            }
        }
        
        going_up = !going_up;
        col = if col >= 2 { col - 2 } else { 0 };
    }
    
    positions
}

/// Check if a position is a function module (finder, timing, format, etc.)
pub fn is_function_module(row: usize, col: usize, size: usize) -> bool {
    // Finder patterns (top-left, top-right, bottom-left)
    if (row < 9 && col < 9) || 
       (row < 9 && col >= size - 8) || 
       (row >= size - 8 && col < 9) {
        return true;
    }
    
    // Timing patterns
    if row == 6 || col == 6 {
        return true;
    }
    
    // Dark module
    if row == 4 * ((size - 17) / 4) + 9 && col == 8 {
        return true;
    }
    
    // Format information areas
    if (row < 9 && (col < 9 || col >= size - 8)) ||
       (row >= size - 8 && col < 9) ||
       (row == 8 && (col < 9 || col >= size - 7)) ||
       (col == 8 && (row < 9 || row >= size - 7)) {
        return true;
    }
    
    // Alignment patterns
    let version = size_to_version(size).unwrap_or(Version::V1);
    let alignment_positions = get_alignment_positions(version);
    for &center_x in &alignment_positions {
        for &center_y in &alignment_positions {
            // Skip if overlaps with finder patterns (same logic as generator)
            if (center_x <= 8 && center_y <= 8) ||
               (center_x <= 8 && center_y >= size - 9) ||
               (center_x >= size - 9 && center_y <= 8) {
                continue;
            }
            
            // Check if current position is within 5x5 alignment pattern
            if row >= center_y.saturating_sub(2) && row <= center_y + 2 &&
               col >= center_x.saturating_sub(2) && col <= center_x + 2 {
                return true;
            }
        }
    }
    
    false
}

/// Convert version enum to size
pub fn version_to_size(version: Version) -> usize {
    match version {
        Version::V1 => 21,
        Version::V2 => 25,
        Version::V3 => 29,
        Version::V4 => 33,
        Version::V5 => 37,
        Version::V6 => 41,
        Version::V7 => 45,
        Version::V8 => 49,
        Version::V9 => 53,
        Version::V10 => 57,
        Version::V11 => 61,
        Version::V12 => 65,
        Version::V13 => 69,
        Version::V14 => 73,
        Version::V15 => 77,
        Version::V16 => 81,
        Version::V17 => 85,
        Version::V18 => 89,
        Version::V19 => 93,
        Version::V20 => 97,
        Version::V21 => 101,
        Version::V22 => 105,
        Version::V23 => 109,
        Version::V24 => 113,
        Version::V25 => 117,
        Version::V26 => 121,
        Version::V27 => 125,
        Version::V28 => 129,
        Version::V29 => 133,
        Version::V30 => 137,
        Version::V31 => 141,
        Version::V32 => 145,
        Version::V33 => 149,
        Version::V34 => 153,
        Version::V35 => 157,
        Version::V36 => 161,
        Version::V37 => 165,
        Version::V38 => 169,
        Version::V39 => 173,
        Version::V40 => 177,
    }
}

/// Convert size to version
pub fn size_to_version(size: usize) -> Option<Version> {
    match size {
        21 => Some(Version::V1),
        25 => Some(Version::V2),
        29 => Some(Version::V3),
        33 => Some(Version::V4),
        37 => Some(Version::V5),
        41 => Some(Version::V6),
        45 => Some(Version::V7),
        49 => Some(Version::V8),
        53 => Some(Version::V9),
        57 => Some(Version::V10),
        61 => Some(Version::V11),
        65 => Some(Version::V12),
        69 => Some(Version::V13),
        73 => Some(Version::V14),
        77 => Some(Version::V15),
        81 => Some(Version::V16),
        85 => Some(Version::V17),
        89 => Some(Version::V18),
        93 => Some(Version::V19),
        97 => Some(Version::V20),
        101 => Some(Version::V21),
        105 => Some(Version::V22),
        109 => Some(Version::V23),
        113 => Some(Version::V24),
        117 => Some(Version::V25),
        121 => Some(Version::V26),
        125 => Some(Version::V27),
        129 => Some(Version::V28),
        133 => Some(Version::V29),
        137 => Some(Version::V30),
        141 => Some(Version::V31),
        145 => Some(Version::V32),
        149 => Some(Version::V33),
        153 => Some(Version::V34),
        157 => Some(Version::V35),
        161 => Some(Version::V36),
        165 => Some(Version::V37),
        169 => Some(Version::V38),
        173 => Some(Version::V39),
        177 => Some(Version::V40),
        _ => None,
    }
}
