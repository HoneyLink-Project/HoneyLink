//! Forward Error Correction strategies

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FecStrategy {
    None,
    Light,
    Heavy,
}

impl FecStrategy {
    pub fn overhead_percent(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Light => 10,
            Self::Heavy => 25,
        }
    }
}
