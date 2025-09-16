use crate::types::{Version, ErrorCorrection, DataMode};

#[allow(dead_code)]
pub fn get_total_codewords(version: Version) -> usize {
    let v = version as u8;
    match v {
        1..=9 => [26, 44, 70, 100, 134, 172, 196, 242, 292][v as usize - 1],
        10..=19 => [346, 404, 466, 532, 581, 655, 733, 815, 901, 991][v as usize - 10],
        20..=29 => [1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051][v as usize - 20],
        30..=40 => [2185, 2323, 2465, 2611, 2761, 2876, 3034, 3196, 3362, 3532, 3706][v as usize - 30],
        _ => panic!("Total codewords not supported for version V{}", v),
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
            _ => panic!("ECC L codewords not supported for version V{}", v),
        },
        ErrorCorrection::M => match v {
            1..=10 => [10, 16, 26, 36, 48, 64, 72, 88, 110, 130][v as usize - 1],
            11..=20 => [150, 176, 198, 216, 240, 280, 308, 338, 364, 416][v as usize - 11],
            21..=30 => [442, 476, 504, 560, 588, 644, 700, 728, 784, 812][v as usize - 21],
            31..=40 => [868, 924, 980, 1036, 1064, 1120, 1204, 1260, 1316, 1372][v as usize - 31],
            _ => panic!("ECC M codewords not supported for version V{}", v),
        },
        ErrorCorrection::Q => match v {
            1..=10 => [13, 22, 36, 52, 72, 96, 108, 132, 160, 192][v as usize - 1],
            11..=20 => [224, 260, 288, 320, 360, 408, 448, 504, 546, 600][v as usize - 11],
            21..=30 => [644, 690, 750, 810, 870, 952, 1020, 1050, 1140, 1200][v as usize - 21],
            31..=40 => [1290, 1350, 1440, 1530, 1590, 1680, 1770, 1860, 1950, 2040][v as usize - 31],
            _ => panic!("ECC Q codewords not supported for version V{}", v),
        },
        ErrorCorrection::H => match v {
            1..=10 => [17, 28, 44, 64, 88, 112, 130, 156, 192, 224][v as usize - 1],
            11..=20 => [264, 308, 352, 384, 432, 480, 532, 588, 650, 700][v as usize - 11],
            21..=30 => [750, 816, 900, 960, 1050, 1110, 1200, 1260, 1350, 1440][v as usize - 21],
            31..=40 => [1530, 1620, 1710, 1800, 1890, 1980, 2100, 2220, 2310, 2430][v as usize - 31],
            _ => panic!("ECC H codewords not supported for version V{}", v),
        },
    }
}

pub fn get_data_capacity(version: Version, error_correction: ErrorCorrection, data_mode: DataMode) -> usize {
    let v = version as u8;
    match (data_mode, error_correction) {
        (DataMode::Numeric, ErrorCorrection::L) => match v {
            1..=10 => [41, 77, 127, 187, 255, 322, 370, 461, 552, 652][v as usize - 1],
            11..=20 => [772, 883, 1022, 1101, 1250, 1408, 1548, 1725, 1903, 2061][v as usize - 11],
            21..=30 => [2232, 2409, 2620, 2812, 3057, 3283, 3517, 3669, 3909, 4158][v as usize - 21],
            31..=40 => [4417, 4686, 4965, 5253, 5529, 5836, 6153, 6479, 6743, 7089][v as usize - 31],
            _ => panic!("Numeric L mode not supported for version V{}", v),
        },
        (DataMode::Numeric, ErrorCorrection::M) => match v {
            1..=10 => [34, 63, 101, 149, 202, 255, 293, 365, 432, 513][v as usize - 1],
            11..=20 => [604, 691, 796, 871, 991, 1082, 1212, 1346, 1500, 1600][v as usize - 11],
            21..=30 => [1708, 1872, 2059, 2188, 2395, 2544, 2701, 2857, 3035, 3289][v as usize - 21],
            31..=40 => [3486, 3693, 3909, 4134, 4343, 4588, 4775, 5039, 5313, 5596][v as usize - 31],
            _ => panic!("Numeric M mode not supported for version V{}", v),
        },
        (DataMode::Numeric, ErrorCorrection::Q) => match v {
            1..=10 => [27, 48, 77, 111, 144, 178, 207, 259, 312, 364][v as usize - 1],
            _ => panic!("Numeric Q mode not supported for version V{}", v),
        },
        (DataMode::Numeric, ErrorCorrection::H) => match v {
            1..=10 => [17, 34, 58, 82, 106, 139, 154, 202, 235, 288][v as usize - 1],
            _ => panic!("Numeric H mode not supported for version V{}", v),
        },
        (DataMode::Alphanumeric, ErrorCorrection::L) => match v {
            1..=10 => [25, 47, 77, 114, 154, 195, 224, 279, 335, 395][v as usize - 1],
            11..=20 => [468, 535, 619, 667, 758, 854, 938, 1046, 1153, 1249][v as usize - 11],
            21..=30 => [1352, 1460, 1588, 1704, 1853, 1990, 2132, 2223, 2369, 2520][v as usize - 21],
            31..=40 => [2677, 2840, 3009, 3183, 3351, 3537, 3729, 3927, 4087, 4296][v as usize - 31],
            _ => panic!("Alphanumeric L mode not supported for version V{}", v),
        },
        (DataMode::Alphanumeric, ErrorCorrection::M) => match v {
            1..=10 => [20, 38, 61, 90, 122, 154, 178, 221, 262, 311][v as usize - 1],
            11..=20 => [366, 419, 483, 528, 600, 656, 734, 816, 909, 970][v as usize - 11],
            21..=30 => [1035, 1134, 1248, 1326, 1451, 1542, 1637, 1732, 1839, 1994][v as usize - 21],
            31..=40 => [2113, 2238, 2369, 2506, 2632, 2780, 2894, 3054, 3220, 3391][v as usize - 31],
            _ => panic!("Alphanumeric M mode not supported for version V{}", v),
        },
        (DataMode::Alphanumeric, ErrorCorrection::Q) => match v {
            1..=10 => [16, 29, 47, 67, 87, 108, 125, 157, 189, 221][v as usize - 1],
            _ => panic!("Alphanumeric Q mode not supported for version V{}", v),
        },
        (DataMode::Alphanumeric, ErrorCorrection::H) => match v {
            1..=10 => [10, 20, 35, 50, 64, 84, 93, 122, 143, 174][v as usize - 1],
            _ => panic!("Alphanumeric H mode not supported for version V{}", v),
        },
        (DataMode::Byte, ErrorCorrection::L) => match v {
            1..=10 => [17, 32, 53, 78, 106, 134, 154, 192, 230, 271][v as usize - 1],
            11..=20 => [321, 367, 425, 458, 520, 586, 644, 718, 792, 858][v as usize - 11],
            21..=30 => [929, 1003, 1091, 1171, 1273, 1367, 1465, 1528, 1628, 1732][v as usize - 21],
            31..=40 => [1840, 1952, 2068, 2188, 2303, 2431, 2563, 2699, 2809, 2953][v as usize - 31],
            _ => panic!("Byte L mode not supported for version V{}", v),
        },
        (DataMode::Byte, ErrorCorrection::M) => match v {
            1..=10 => [14, 26, 42, 62, 84, 106, 122, 152, 180, 213][v as usize - 1],
            11..=20 => [251, 287, 331, 362, 412, 450, 504, 560, 624, 666][v as usize - 11],
            21..=30 => [711, 779, 857, 911, 997, 1059, 1125, 1190, 1264, 1370][v as usize - 21],
            31..=40 => [1452, 1538, 1628, 1722, 1809, 1911, 1989, 2099, 2213, 2331][v as usize - 31],
            _ => panic!("Byte M mode not supported for version V{}", v),
        },
        (DataMode::Byte, ErrorCorrection::Q) => match v {
            1..=10 => [11, 20, 32, 46, 60, 74, 86, 108, 130, 151][v as usize - 1],
            _ => panic!("Byte Q mode not supported for version V{}", v),
        },
        (DataMode::Byte, ErrorCorrection::H) => match v {
            1..=10 => [7, 14, 24, 34, 44, 58, 64, 84, 98, 119][v as usize - 1],
            _ => panic!("Byte H mode not supported for version V{}", v),
        },
    }
}
