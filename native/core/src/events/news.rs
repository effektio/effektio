use super::{
    Colorize, ImageMessageEventContent, TextMessageEventContent, VideoMessageEventContent,
};
use ruma::events::macros::EventContent;
use matrix_sdk_base::deserialized_responses::SyncRoomEvent;
use serde::{Deserialize, Serialize};

/// The content that is specific to each message type variant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NewsContentType {
    /// An image message.
    Image(ImageMessageEventContent),
    /// A text message.
    Text(TextMessageEventContent),
    /// A video message.
    Video(VideoMessageEventContent),
}

/// The payload for our news event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.effektio.news.dev", kind = MessageLike)]
pub struct NewsEventDevContent {
    pub contents: Vec<NewsContentType>,
    pub colors: Option<Colorize>,
}

/// The content that is specific to each news type variant.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NewsEvent {
    Dev(ruma::events::MessageLikeEvent<NewsEventDevContent>),
}

impl NewsEvent {
    pub fn to_model(&self) -> anyhow::Result<crate::models::EffektioModel> {
        let content = if let NewsEvent::Dev(ruma::events::MessageLikeEvent::Original(inner)) = self {
            &inner.content
        } else  {
            anyhow::bail!("not the proper event");
        };
        let (bg_color, fg_color) = if let Some(color) = &content.colors {
            (color.background.clone(), color.color.clone())
        } else {
            (None, None)
        };
        let text = content.contents.iter().find_map(|m| { if let NewsContentType::Text(t) = m { Some(t.body.clone())} else { None } });
        Ok(crate::models::EffektioModel::News(
            crate::models::News {
                text,
                bg_color,
                fg_color,
                comments_count: 0,
                likes_count: 0,
                image: None,
                tags: Default::default(),
            }
        ))

    }
}

impl TryFrom<&SyncRoomEvent> for NewsEvent {
    type Error = anyhow::Error;

    fn try_from(other: &SyncRoomEvent) -> Result<Self, Self::Error> {
        Ok(NewsEvent::Dev(other.event.deserialize_as()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matrix_sdk_base::deserialized_responses::SyncRoomEvent;
    use ruma::{room_id, events::OriginalSyncMessageLikeEvent};

    #[test]
    fn parse_test() {
        let json = serde_json::json!({
            "content": {
                "contents": [ {
                    "type": "text",
                    "body": "This is an important news"
                }],
            },
            "event_id": "$12345-asdf",
            "origin_server_ts": 1u64,
            "sender": "@someone:example.org",
            "type": "org.effektio.news.dev",
            "unsigned": {
                "age": 85u64
            }
        });

        serde_json::from_value::<OriginalSyncMessageLikeEvent<NewsEventDevContent>>(json).unwrap();
    }

    #[test]
    fn sync_room_event_parse() {
        let json = serde_json::json!({
            "event": {
                "content": {
                    "contents": [ {
                        "type": "text",
                        "body": "This is an important news"
                    }],
                },
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

        let inner: OriginalSyncMessageLikeEvent<NewsEventDevContent> = sync_room.event.deserialize_as().unwrap();
    }

    #[test]
    fn sync_room_event_meta_parse() {
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
            },
        });

        let sync_room = serde_json::from_value::<SyncRoomEvent>(json).unwrap();

        let inner: NewsEvent = (&sync_room).try_into().unwrap();

    }
}