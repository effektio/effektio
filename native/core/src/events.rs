pub use ruma::{
    events::{
        room::message::{
            ImageMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
        },
        AnyMessageLikeEvent,
    },
    EventId,
};
use matrix_sdk_base::deserialized_responses::SyncRoomEvent;
use serde::{Deserialize, Serialize};

mod comments;
mod common;
mod labels;
mod news;
mod todos;

pub use comments::{CommentEvent, CommentEventDevContent};
pub use common::{BelongsTo, Color, Colorize, TimeZone, UtcDateTime};
pub use labels::Labels;
pub use news::{NewsContentType, NewsEvent, NewsEventDevContent};
pub use todos::{
    Priority as TaskPriority, SpecialTaskListRole, Task, TaskDevContent, TaskList,
    TaskListDevContent,
};

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum AnyEffektioMessageLikeEvent {
    News(NewsEvent),
    Matrix(AnyMessageLikeEvent)
}


impl TryFrom<&SyncRoomEvent> for AnyEffektioMessageLikeEvent {
    type Error = anyhow::Error;

    fn try_from(other: &SyncRoomEvent) -> Result<Self, Self::Error> {
        let msg_type = if let Some(ev) = other.event.get_field::<String>("type")? {
            ev
        } else  {
            anyhow::bail!("Can't convert SyncRoomEvent: missing `type` field");
        };

        Ok(if msg_type.starts_with("org.effektio.news.") {
            AnyEffektioMessageLikeEvent::News(other.try_into()?)
        } else {
            AnyEffektioMessageLikeEvent::Matrix(
                other.event.deserialize_as::<AnyMessageLikeEvent>()?
            )
        })
    }
}



#[cfg(test)]
mod integration {
    use super::*;
    use assert_matches::assert_matches;
    use matrix_sdk_base::deserialized_responses::SyncRoomEvent;
    use ruma::{room_id, events::OriginalSyncMessageLikeEvent};

    #[test]
    fn sync_room_news_event_parse() {
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

        let effektio_event: AnyEffektioMessageLikeEvent = (&sync_room).try_into().unwrap();
        assert_matches!(effektio_event, AnyEffektioMessageLikeEvent::News(_));

    }
}