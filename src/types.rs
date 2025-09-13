#[derive(Clone, Copy, Debug)]
pub enum Version {
    V1, V2, V3, V4, V5, V6, V7, V8, V9, V10,
    V11, V12, V13, V14, V15, V16, V17, V18, V19, V20,
    V21, V22, V23, V24, V25, V26, V27, V28, V29, V30,
    V31, V32, V33, V34, V35, V36, V37, V38, V39, V40,
}

impl Version {
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
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorCorrection {
    L, // Low (~7%)
    M, // Medium (~15%)
    Q, // Quartile (~25%)
    H, // High (~30%)
}

#[derive(Clone, Copy, Debug)]
pub enum DataMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}

#[derive(Clone, Copy, Debug)]
pub enum MaskPattern {
    Pattern0, Pattern1, Pattern2, Pattern3,
    Pattern4, Pattern5, Pattern6, Pattern7,
}

pub struct QrConfig {
    pub version: Version,
    pub error_correction: ErrorCorrection,
    pub data_mode: DataMode,
    pub mask_pattern: MaskPattern,
    pub skip_mask: bool,
    pub output_filename: String,
    pub url: String,
    pub verbose: bool,
}

impl Default for QrConfig {
    fn default() -> Self {
        Self {
            version: Version::V3,
            error_correction: ErrorCorrection::H,
            data_mode: DataMode::Byte,
            mask_pattern: MaskPattern::Pattern0,
            skip_mask: false,
            output_filename: "qr-code.png".to_string(),
            url: "https://www.example.com/".to_string(),
            verbose: false,
        }
    }
}

pub type QrMatrix = [[u8; 29]; 29];
