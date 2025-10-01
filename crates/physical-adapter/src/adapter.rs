//! Physical adapter implementation

pub struct PhysicalAdapter {
    adapter_type: AdapterType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterType {
    WiFi6E,
    WiFi7,
    FiveG,
    THz,
    Bluetooth,
}

impl PhysicalAdapter {
    pub fn new(adapter_type: AdapterType) -> Self {
        Self { adapter_type }
    }

    pub fn adapter_type(&self) -> AdapterType {
        self.adapter_type
    }
}
