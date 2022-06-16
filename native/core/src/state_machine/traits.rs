use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait Action: Clone + Debug {}

/// Describing the state transiiton that action can produce.
pub trait Transition: Serialize + DeserializeOwned + Debug {
    type Action: Action;

    /// Apply the given action on the object
    /// Return value indicates whether state has actually changed during the transition
    fn transition(&mut self, action: Self::Action) -> Result<bool>;
}

/// Simple way to configure support for generic features
pub trait GenericFeaturesSupport {
    /// Whether you can redact this entity,
    /// defaults to true
    fn support_redaction(&self) -> bool {
        true
    }

    /// Whether this support comments
    /// defaults to false
    fn supports_comments(&self) -> bool {
        false
    }

    /// Whether you can react to this entity
    /// defaults to false
    fn supports_reactions(&self) -> bool {
        false
    }
}
