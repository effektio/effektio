pub use chrono::{DateTime, Utc};
pub use chrono_tz::Tz as TimeZone;
pub use csscolorparser::Color;
use matrix_sdk::ruma::OwnedEventId;
use serde::{Deserialize, Serialize};

/// Default UTC Datetime Object
pub type UtcDateTime = DateTime<Utc>;

/// Customize the color scheme
#[derive(Clone, Debug, Deserialize, Serialize)]
// #[ruma_event(type = "org.effektio.dev.colors")]
pub struct Colorize {
    /// The foreground color to be used, as HEX
    pub color: Option<Color>,
    /// The background color to be used, as HEX
    pub background: Option<Color>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "rel_type", rename = "m.thread")]
pub struct InThread {
    /// The event this event archives.
    pub event_id: OwnedEventId,
}

impl From<OwnedEventId> for InThread {
    fn from(event_id: OwnedEventId) -> InThread {
        InThread { event_id }
    }
}

pub type BelongsTo = InThread;
