//! QoS scheduler implementation

use honeylink_core::types::StreamId;

pub struct QoSScheduler {
    streams: Vec<StreamId>,
}

impl QoSScheduler {
    pub fn new() -> Self {
        Self { streams: Vec::new() }
    }

    pub fn add_stream(&mut self, stream_id: StreamId) {
        self.streams.push(stream_id);
    }
}

impl Default for QoSScheduler {
    fn default() -> Self {
        Self::new()
    }
}
