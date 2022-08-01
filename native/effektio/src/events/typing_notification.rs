use futures::channel::mpsc::Sender;
use matrix_sdk::{
    ruma::{events::AnySyncEphemeralRoomEvent, OwnedRoomId},
    Client,
};

#[derive(Clone, Debug)]
pub struct TypingNotification {
    room_id: String,
}

impl TypingNotification {
    pub(crate) fn new(room_id: String) -> Self {
        Self { room_id }
    }

    pub fn get_room_id(&self) -> String {
        self.room_id.clone()
    }
}

// thread callback must be global function, not member function
pub async fn handle_typing_notification(
    room_id: &OwnedRoomId,
    event: &AnySyncEphemeralRoomEvent,
    client: &Client,
    tx: &mut Sender<TypingNotification>,
) {
    if let AnySyncEphemeralRoomEvent::Typing(ev) = event {
        let evt = TypingNotification::new(room_id.to_string());
        if let Err(e) = tx.try_send(evt) {
            log::warn!("Dropping ephemeral event for {}: {}", room_id, e);
        }
    }
}
