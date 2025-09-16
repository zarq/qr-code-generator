#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[allow(dead_code)]
pub enum Version {
    V1 = 1, V2, V3, V4, V5, V6, V7, V8, V9, V10,
    V11, V12, V13, V14, V15, V16, V17, V18, V19, V20,
    V21, V22, V23, V24, V25, V26, V27, V28, V29, V30,
    V31, V32, V33, V34, V35, V36, V37, V38, V39, V40,
}

impl Version {
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        match self {
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
            _ => 21 + ((*self as usize) * 4),
        }
    }

    #[allow(dead_code)]
    pub fn from_u8(n: u8) -> Option<Version> {
        match n {
            1 => Some(Version::V1), 2 => Some(Version::V2), 3 => Some(Version::V3), 4 => Some(Version::V4), 5 => Some(Version::V5),
            6 => Some(Version::V6), 7 => Some(Version::V7), 8 => Some(Version::V8), 9 => Some(Version::V9), 10 => Some(Version::V10),
            11 => Some(Version::V11), 12 => Some(Version::V12), 13 => Some(Version::V13), 14 => Some(Version::V14), 15 => Some(Version::V15),
            16 => Some(Version::V16), 17 => Some(Version::V17), 18 => Some(Version::V18), 19 => Some(Version::V19), 20 => Some(Version::V20),
            21 => Some(Version::V21), 22 => Some(Version::V22), 23 => Some(Version::V23), 24 => Some(Version::V24), 25 => Some(Version::V25),
            26 => Some(Version::V26), 27 => Some(Version::V27), 28 => Some(Version::V28), 29 => Some(Version::V29), 30 => Some(Version::V30),
            31 => Some(Version::V31), 32 => Some(Version::V32), 33 => Some(Version::V33), 34 => Some(Version::V34), 35 => Some(Version::V35),
            36 => Some(Version::V36), 37 => Some(Version::V37), 38 => Some(Version::V38), 39 => Some(Version::V39), 40 => Some(Version::V40),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum ErrorCorrection {
    L, // Low (~7%)
    M, // Medium (~15%)
    Q, // Quartile (~25%)
    H, // High (~30%)
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum DataMode {
    Numeric,
    Alphanumeric,
    Byte,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum MaskPattern {
    Pattern0, Pattern1, Pattern2, Pattern3,
    Pattern4, Pattern5, Pattern6, Pattern7,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OutputFormat {
    Png,
    Svg,
}

#[allow(dead_code)]
pub struct QrConfig {
    pub error_correction: ErrorCorrection,
    pub data_mode: DataMode,
    pub mask_pattern: MaskPattern,
    pub skip_mask: bool,
    pub output_filename: String,
    pub output_format: OutputFormat,
    pub data: String,
    pub verbose: bool,
}

impl Default for QrConfig {
    fn default() -> Self {
        Self {
            error_correction: ErrorCorrection::M,
            data_mode: DataMode::Byte,
            mask_pattern: MaskPattern::Pattern0,
            skip_mask: false,
            output_filename: "qr-code.png".to_string(),
            output_format: OutputFormat::Png,
            data: "https://www.example.com/".to_string(),
            verbose: false,
        }
    }
}
