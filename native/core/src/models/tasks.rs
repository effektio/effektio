use crate::events;
use matrix_sdk::ruma::{events::OriginalMessageLikeEvent, OwnedEventId, OwnedRoomId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskList {
    inner: OriginalMessageLikeEvent<events::TaskListContent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    inner: OriginalMessageLikeEvent<events::TaskContent>,
    tasks: Vec<TaskList>,
}

impl TaskList {
    pub fn redacted(&self) -> bool {
        false
    }
    pub fn event_id(&self) -> OwnedEventId {
        self.inner.event_id.clone()
    }
    pub fn room_id(&self) -> OwnedRoomId {
        self.inner.room_id.clone()
    }
    pub fn name(&self) -> Option<String> {
        Some(self.inner.content.name.clone())
    }
}
