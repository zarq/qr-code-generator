use crate::types::{Version, ErrorCorrection, DataMode};

#[allow(dead_code)]
pub fn get_total_codewords(version: Version) -> usize {
    let v = version as u8;
    match v {
        1..=9 => [26, 44, 70, 100, 134, 172, 196, 242, 292][v as usize - 1],
        10..=19 => [346, 404, 466, 532, 581, 655, 733, 815, 901, 991][v as usize - 10],
        20..=29 => [1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051][v as usize - 20],
        30..=40 => [2185, 2323, 2465, 2611, 2761, 2876, 3034, 3196, 3362, 3532, 3706][v as usize - 30],
        _ => 0,
    }
}

#[allow(dead_code)]
pub fn get_ecc_codewords(version: Version, error_correction: ErrorCorrection) -> usize {
    let v = version as u8;
    match error_correction {
        ErrorCorrection::L => match v {
            1..=10 => [7, 10, 15, 20, 26, 36, 40, 48, 60, 72][v as usize - 1],
            11..=20 => [80, 96, 104, 120, 132, 144, 168, 180, 196, 224][v as usize - 11],
            21..=30 => [224, 252, 270, 300, 312, 336, 360, 390, 420, 450][v as usize - 21],
            31..=40 => [480, 510, 540, 570, 570, 600, 630, 660, 720, 750][v as usize - 31],
            _ => 0,
        },
        ErrorCorrection::M => match v {
            1..=10 => [10, 16, 26, 36, 48, 64, 72, 88, 110, 130][v as usize - 1],
            11..=20 => [150, 176, 198, 216, 240, 280, 308, 338, 364, 416][v as usize - 11],
            21..=30 => [442, 476, 504, 560, 588, 644, 700, 728, 784, 812][v as usize - 21],
            31..=40 => [868, 924, 980, 1036, 1064, 1120, 1204, 1260, 1316, 1372][v as usize - 31],
            _ => 0,
        },
        ErrorCorrection::Q => match v {
            1..=10 => [13, 22, 36, 52, 72, 96, 108, 132, 160, 192][v as usize - 1],
            11..=20 => [224, 260, 288, 320, 360, 408, 448, 504, 546, 600][v as usize - 11],
            21..=30 => [644, 690, 750, 810, 870, 952, 1020, 1050, 1140, 1200][v as usize - 21],
            31..=40 => [1290, 1350, 1440, 1530, 1590, 1680, 1770, 1860, 1950, 2040][v as usize - 31],
            _ => 0,
        },
        ErrorCorrection::H => match v {
            1..=10 => [17, 28, 44, 64, 88, 112, 130, 156, 192, 224][v as usize - 1],
            11..=20 => [264, 308, 352, 384, 432, 480, 532, 588, 650, 700][v as usize - 11],
            21..=30 => [750, 816, 900, 960, 1050, 1110, 1200, 1260, 1350, 1440][v as usize - 21],
            31..=40 => [1530, 1620, 1710, 1800, 1890, 1980, 2100, 2220, 2310, 2430][v as usize - 31],
            _ => 0,
        },
    }
}

pub fn get_data_capacity(version: Version, error_correction: ErrorCorrection, data_mode: DataMode) -> usize {
    let v = version as u8;
    match (data_mode, error_correction) {
        (DataMode::Numeric, ErrorCorrection::L) => match v {
            1..=10 => [41, 77, 127, 187, 255, 322, 370, 461, 552, 652][v as usize - 1],
            _ => 0,
        },
        (DataMode::Numeric, ErrorCorrection::M) => match v {
            1..=10 => [34, 63, 101, 149, 202, 255, 293, 365, 432, 513][v as usize - 1],
            _ => 0,
        },
        (DataMode::Numeric, ErrorCorrection::Q) => match v {
            1..=10 => [27, 48, 77, 111, 144, 178, 207, 259, 312, 364][v as usize - 1],
            _ => 0,
        },
        (DataMode::Numeric, ErrorCorrection::H) => match v {
            1..=10 => [17, 34, 58, 82, 106, 139, 154, 202, 235, 288][v as usize - 1],
            _ => 0,
        },
        (DataMode::Alphanumeric, ErrorCorrection::L) => match v {
            1..=10 => [25, 47, 77, 114, 154, 195, 224, 279, 335, 395][v as usize - 1],
            _ => 0,
        },
        (DataMode::Alphanumeric, ErrorCorrection::M) => match v {
            1..=10 => [20, 38, 61, 90, 122, 154, 178, 221, 262, 311][v as usize - 1],
            _ => 0,
        },
        (DataMode::Alphanumeric, ErrorCorrection::Q) => match v {
            1..=10 => [16, 29, 47, 67, 87, 108, 125, 157, 189, 221][v as usize - 1],
            _ => 0,
        },
        (DataMode::Alphanumeric, ErrorCorrection::H) => match v {
            1..=10 => [10, 20, 35, 50, 64, 84, 93, 122, 143, 174][v as usize - 1],
            _ => 0,
        },
        (DataMode::Byte, ErrorCorrection::L) => match v {
            1..=10 => [17, 32, 53, 78, 106, 134, 154, 192, 230, 271][v as usize - 1],
            _ => 0,
        },
        (DataMode::Byte, ErrorCorrection::M) => match v {
            1..=10 => [14, 26, 42, 62, 84, 106, 122, 152, 180, 213][v as usize - 1],
            _ => 0,
        },
        (DataMode::Byte, ErrorCorrection::Q) => match v {
            1..=10 => [11, 20, 32, 46, 60, 74, 86, 108, 130, 151][v as usize - 1],
            _ => 0,
        },
        (DataMode::Byte, ErrorCorrection::H) => match v {
            1..=10 => [7, 14, 24, 34, 44, 58, 64, 84, 98, 119][v as usize - 1],
            _ => 0,
        },
    }
}
