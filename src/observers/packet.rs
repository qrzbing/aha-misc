//! PacketObserver

use std::borrow::Cow;

use libafl::{Error, observers::Observer};
use libafl_bolts::Named;
use serde::{Deserialize, Serialize};

/// PacketObserver gets the latest packet.
#[derive(Debug, Serialize, Deserialize)]
pub struct PacketObserver {
    /// Observer Name
    name: Cow<'static, str>,
    /// Last packet, None if not received
    last_packet: Option<Vec<u8>>,
}

impl PacketObserver {
    /// Create a new PacketObserver
    pub fn new<S>(name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            last_packet: None,
        }
    }

    /// Clear status by Executor
    pub fn clear(&mut self) {
        self.last_packet = None;
    }

    /// Set packet by Executor
    pub fn set_packet(&mut self, packet: Vec<u8>) {
        self.last_packet = Some(packet);
    }

    /// Get last packet
    pub fn last_packet(&self) -> Option<&Vec<u8>> {
        self.last_packet.as_ref()
    }
}

impl Named for PacketObserver {
    fn name(&self) -> &Cow<'static, str> {
        &self.name
    }
}

impl<I, S> Observer<I, S> for PacketObserver {
    fn post_exec(
        &mut self,
        _state: &mut S,
        _input: &I,
        _exit_kind: &libafl::executors::ExitKind,
    ) -> Result<(), Error> {
        Ok(())
    }
}
