use matrix_sdk_base::{deserialized_responses::SyncRoomEvent, store::StateStore};
use crate::events::{AnyEffektioMessageLikeEvent, NewsEvent};
use ruma::{
    events::{
        OriginalMessageLikeEvent, AnyMessageLikeEvent, MessageLikeEvent,
        reaction::ReactionEventContent,
    },
    OwnedEventId, OwnedRoomId
};
use crate::store::Store;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Executor {
    store: Arc<Store>,
}

impl Executor {
    pub fn new(store: Arc<Store>) -> Self {
        Executor { store }
    }

    pub async fn apply(&self, event: &SyncRoomEvent) -> anyhow::Result<Option<OwnedEventId>> {
        let effektio_event: AnyEffektioMessageLikeEvent = event.try_into()?;

        Ok(match effektio_event {
            AnyEffektioMessageLikeEvent::Matrix(m) => {
                self.handle_regular_matrix_message(m).await?
            }
            AnyEffektioMessageLikeEvent::News(NewsEvent::Dev(n)) => {
                unimplemented!()
                
            }
        })
    }
}

// Regular Matrix Message System

impl Executor {

    pub async fn handle_regular_matrix_message(&self, event: AnyMessageLikeEvent) -> anyhow::Result<Option<OwnedEventId>> {
        match event {
            AnyMessageLikeEvent::RoomMessage(ml) => {
                match ml {
                    MessageLikeEvent::Original(m) => {
                        unimplemented!()
                        // creates a new entry
                    }
                    MessageLikeEvent::Redacted(m) => {
                        unimplemented!()
                        // creates a new entry
                    }

                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub async fn handle_reaction(&self, event: OriginalMessageLikeEvent<ReactionEventContent>) -> anyhow::Result<()> {
        unimplemented!()

    } 
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::events::{NewsEvent, NewsEventDevContent};
    use ruma::{room_id, event_id};
    use std::sync::Arc;
    use crate::store::test_helpers::test_store;

    fn new_executor() -> Executor {
        Executor::new(Arc::new(test_store()))
    }

    #[tokio::test]
    async fn smoke_test() {
        let exec = new_executor();
    }


    #[tokio::test]
    async fn news_event() {
        let exec = new_executor();
        let json = serde_json::json!({
            "event": {
                "content": {
                    "contents": [ {
                        "type": "text",
                        "body": "This is an important news"
                    }],
                },
                "room_id": "!whatev:example.org",
                "event_id": "$12345-asdf",
                "origin_server_ts": 1u64,
                "sender": "@someone:example.org",
                "type": "org.effektio.news.dev",
                "unsigned": {
                    "age": 85u64
                }
            }
        });

        let sync_room = serde_json::from_value::<SyncRoomEvent>(json).unwrap();
        assert_eq!(exec.apply(&sync_room).await.unwrap().unwrap(),
            event_id!("$12345-asdf").to_owned())

    }
}