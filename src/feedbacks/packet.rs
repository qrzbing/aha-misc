//! PacketFeedback

use crate::observers::packet::PacketObserver;
use libafl::{
    Error,
    executors::ExitKind,
    feedbacks::{Feedback, StateInitializer},
};
use libafl_bolts::{
    Named,
    tuples::{Handle, Handled, MatchName, MatchNameRef},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashSet};

/// PacketFeedback
#[derive(Debug, Serialize, Deserialize)]
pub struct PacketFeedback {
    observer_handle: Handle<PacketObserver>,
    #[serde(skip)]
    history: HashSet<Vec<u8>>,
}

impl PacketFeedback {
    /// Creates a new PacketFeedback from a Handle to a PacketObserver.
    pub fn new(observer: &PacketObserver) -> Self {
        Self {
            observer_handle: observer.handle(),
            history: HashSet::new(),
        }
    }
}

impl<S> StateInitializer<S> for PacketFeedback {}

impl<EM, I, OT, S> Feedback<EM, I, OT, S> for PacketFeedback
where
    OT: MatchName,
{
    fn is_interesting(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &I,
        observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error> {
        let Some(observer) = observers.get(&self.observer_handle) else {
            return Err(Error::illegal_state(format!(
                "Observer {:?} not found in the ObserversTuple",
                self.observer_handle
            )));
        };

        let Some(packet) = observer.last_packet() else {
            return Ok(false);
        };

        if self.history.insert(packet.clone()) {
            return Ok(true);
        }
        return Ok(false);
    }
}

impl Named for PacketFeedback {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("PacketFeedback");
        &NAME
    }
}
