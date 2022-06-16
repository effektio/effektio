use anyhow::{bail,  Result};
use serde::{Deserialize, Serialize,};

use std::fmt::Debug;

mod binary_switch;
mod reaction;
mod traits;

pub use binary_switch::{BinarySwitch, BinarySwitchAction};
pub use reaction::ReactionState;
pub use traits::{Action, GenericFeaturesSupport, Transition};

use ruma::{
    events::{reaction::ReactionEvent, room::redaction::RoomRedactionEvent},
    EventId, MilliSecondsSinceUnixEpoch, OwnedUserId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: Transition")]
pub enum InnerState<T> {
    Redacted {
        reason: Option<String>,
        sender: OwnedUserId,
        when: MilliSecondsSinceUnixEpoch,
    },
    Alive(T),
    Archived {
        sender: OwnedUserId,
        when: MilliSecondsSinceUnixEpoch,
        obj: T,
    },
}

impl<T> InnerState<T> {
    fn inner(&self) -> Option<&T> {
        match self {
            InnerState::Alive(ref o) => Some(o),
            InnerState::Archived { ref obj, .. } => Some(obj),
            InnerState::Redacted { .. } => None,
        }
    }
}

impl<T> Transition for InnerState<T>
where
    T: Transition,
{
    type Action = <T as Transition>::Action;
    fn transition(&mut self, action: Self::Action) -> Result<bool> {
        match self {
            InnerState::Alive(o) => o.transition(action),
            InnerState::Archived { obj, .. } => obj.transition(action),
            InnerState::Redacted { .. } => {
                bail!("Has been redacted. Can't transition on redacted object.")
            }
        }
    }
}

impl<T> GenericFeaturesSupport for InnerState<T>
where
    T: GenericFeaturesSupport,
{
    fn support_redaction(&self) -> bool {
        self.inner()
            .map(|o| o.support_redaction())
            .unwrap_or_default()
    }

    fn supports_comments(&self) -> bool {
        self.inner()
            .map(|o| o.supports_comments())
            .unwrap_or_default()
    }

    fn supports_reactions(&self) -> bool {
        self.inner()
            .map(|o| o.supports_reactions())
            .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: Transition")]
pub struct StatefulObject<T>
where
    T: Transition,
{
    inner: InnerState<T>,

    #[serde(default)]
    #[serde(skip_serializing_if = "ReactionState::is_empty")]
    reactions: ReactionState,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    history: Vec<Box<EventId>>,
}

impl<T: Transition> StatefulObject<T> {
    pub fn new(state: T) -> Self {
        StatefulObject {
            inner: InnerState::Alive(state),
            reactions: ReactionState(Default::default()),
            history: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GenericAction<A: Action> {
    Redaction(RoomRedactionEvent),
    // Archive(String),
    Reaction(ReactionEvent),
    SpecificAction(A),
}

impl<T: Action> Action for GenericAction<T> {}

impl<T> From<RoomRedactionEvent> for GenericAction<T>
where
    T: Action,
{
    fn from(other: RoomRedactionEvent) -> Self {
        GenericAction::Redaction(other)
    }
}

impl<T> From<ReactionEvent> for GenericAction<T>
where
    T: Action,
{
    fn from(other: ReactionEvent) -> Self {
        GenericAction::Reaction(other)
    }
}

impl<T> Transition for StatefulObject<T>
where
    T: Transition + GenericFeaturesSupport,
{
    type Action = GenericAction<<T as Transition>::Action>;
    fn transition(&mut self, action: Self::Action) -> Result<bool> {
        if matches!(self.inner, InnerState::Redacted { .. }) {
            bail!("Object has been redacted already");
        }

        Ok(match action {
            GenericAction::Reaction(r) => {
                if !self.inner.supports_reactions() {
                    bail!("Reacting not supported");
                }
                self.reactions.transition(r)?
            }
            GenericAction::SpecificAction(a) => self.inner.transition(a)?,
            // FIXME
            _ => false,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    #[test]
    fn smoketest() -> Result<()> {
        let mut m = StatefulObject::new(BinarySwitch::new());

        m.transition(BinarySwitchAction::SwitchOn.into())?;
        let on_json = serde_json::to_string(&m)?;
        println!("After on: {:}", on_json);

        m.transition(BinarySwitchAction::SwitchOff.into())?;
        let off_json = serde_json::to_string(&m)?;
        println!("After off: {:}", off_json);

        let mut recov = serde_json::from_str::<StatefulObject<BinarySwitch>>(&on_json)?;
        println!("Recovered State: {:?}", recov);

        recov.transition(BinarySwitchAction::SwitchOff.into())?;
        println!("Recovered State off: {:?}", recov);

        // recov.transition(GenericAction::Redaction(Redacted{by: "Ben".to_owned(), reason: None}))?;
        // let redacetd = serde_json::to_string(&recov)?;
        // println!("Redacted State: {:?}", redacetd);

        assert!(recov
            .transition(BinarySwitchAction::SwitchOn.into())
            .is_err());
        Ok(())
    }
}
