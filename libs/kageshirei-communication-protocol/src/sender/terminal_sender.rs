use std::sync::Arc;

use bytes::Bytes;

use super::Sender;
use crate::metadata::Metadata;

pub struct TerminalSender {
    is_checkin: bool,
}

impl Default for TerminalSender {
    fn default() -> Self { Self::new() }
}

impl TerminalSender {
    pub const fn new() -> Self {
        Self {
            is_checkin: false,
        }
    }
}

impl Sender for TerminalSender {
    fn set_is_checkin(&mut self, is_checkin: bool) -> &Self {
        self.is_checkin = is_checkin;

        self
    }

    async fn send(&mut self, data: Bytes, _metadata: Arc<Metadata>) -> Result<Bytes, String> {
        println!("{:?}", data);
        Ok(data)
    }
}
