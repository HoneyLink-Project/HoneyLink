//! Weighted Fair Queuing scheduler

pub struct WeightedFairQueuing {
    weights: [u8; 3], // [control, data, telemetry]
}

impl WeightedFairQueuing {
    pub fn new() -> Self {
        Self {
            weights: [25, 60, 15], // Default allocation
        }
    }

    pub fn set_weights(&mut self, control: u8, data: u8, telemetry: u8) {
        self.weights = [control, data, telemetry];
    }
}

impl Default for WeightedFairQueuing {
    fn default() -> Self {
        Self::new()
    }
}
