use anyhow::{Result};
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use super::{Action, GenericAction, GenericFeaturesSupport, Transition};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinarySwitch(BinarySwitchState);

impl BinarySwitch {
    pub fn new() -> Self {
        BinarySwitch(BinarySwitchState::Off)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinarySwitchState {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinarySwitchAction {
    SwitchOn,
    SwitchOff,
}

impl Into<GenericAction<BinarySwitchAction>> for BinarySwitchAction {
    fn into(self) -> GenericAction<BinarySwitchAction> {
        GenericAction::SpecificAction(self)
    }
}

impl Action for BinarySwitchAction {}

impl GenericFeaturesSupport for BinarySwitch {}

impl Transition for BinarySwitch {
    type Action = BinarySwitchAction;
    fn transition(&mut self, action: Self::Action) -> Result<bool> {
        self.0 = match action {
            BinarySwitchAction::SwitchOn => BinarySwitchState::On,
            BinarySwitchAction::SwitchOff => BinarySwitchState::Off,
        };
        Ok(true)
    }
}
