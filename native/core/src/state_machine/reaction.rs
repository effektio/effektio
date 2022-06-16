use anyhow::Result;
use serde::{
    Deserialize, Serialize,
};

use super::{Action, Transition};
use ruma::{events::reaction::ReactionEvent, events::MessageLikeEvent, OwnedUserId};
use std::collections::btree_map::{BTreeMap, Entry};
use std::fmt::Debug;

pub type ReactionMap = BTreeMap<String, Vec<OwnedUserId>>;
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ReactionState(pub ReactionMap);

impl core::ops::Deref for ReactionState {
    type Target = ReactionMap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ReactionState {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Action for ReactionEvent {}

impl Transition for ReactionState {
    type Action = ReactionEvent;
    fn transition(&mut self, action: Self::Action) -> Result<bool> {
        let event = match action {
            MessageLikeEvent::Original(u) => u,
            MessageLikeEvent::Redacted(_) => {
                // FIXME: not yet supported
                return Ok(false);
            }
        };
        match self.0.entry(event.content.relates_to.key) {
            Entry::Vacant(o) => {
                o.insert(vec![event.sender]);
                Ok(true)
            }
            Entry::Occupied(mut o) => {
                let users = o.get_mut();
                let sender = event.sender;
                if !users.contains(&sender) {
                    // we ignore if the user is already in the list
                    users.push(sender);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
