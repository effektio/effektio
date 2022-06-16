use super::TextMessageEventContent;
use ruma::events::macros::EventContent;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// ToDo Lists and Task Items management
/// modeled after [JMAP Tasks](https://jmap.io/spec-tasks.html), extensions to
/// [ietf rfc8984](https://www.rfc-editor.org/rfc/rfc8984.html#name-task).
///
use super::{BelongsTo, Color, TimeZone, UtcDateTime};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SpecialTaskListRole {
    Inbox,
    Trash,
}

#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug)]
#[repr(u8)]
/// Implementing Priority according to
/// <https://www.rfc-editor.org/rfc/rfc8984.html#section-4.4.1>
pub enum Priority {
    Undefined = 0,
    Highest = 1,
    SecondHighest = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    SecondLowest = 8,
    Lowest = 9,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Undefined
    }
}

/// The TaskList Event
///
/// modeled after [JMAP TaskList](https://jmap.io/spec-tasks.html#tasklists)
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.effektio.dev.tasklist", kind = MessageLike)]
pub struct TaskListDevContent {
    pub role: Option<SpecialTaskListRole>,
    pub name: String,
    pub description: Option<TextMessageEventContent>,
    pub color: Option<Color>,
    pub sort_order: u32,
    pub time_zone: Option<TimeZone>,
    // FIXME: manage through `label` as in [MSC2326](https://github.com/matrix-org/matrix-doc/pull/2326)
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

/// The Task Event
///
/// modeled after [JMAP Task](https://jmap.io/spec-tasks.html#tasks)
/// see also the [IETF Task](https://www.rfc-editor.org/rfc/rfc8984.html#name-task)
/// but all timezones have been dumbed down to UTC-only.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.effektio.dev.task", kind = MessageLike)]
pub struct TaskDevContent {
    #[serde(rename = "m.relates_to")]
    pub task_list_id: BelongsTo,
    pub title: String,
    pub description: Option<TextMessageEventContent>,
    pub utc_due: Option<UtcDateTime>,
    pub utc_start: Option<UtcDateTime>,
    pub sort_order: u32,
    pub color: Option<Color>,
    // FIXME: manage through `label` as in [MSC2326](https://github.com/matrix-org/matrix-doc/pull/2326)
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

/// The content that is specific to each news type variant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Task {
    Dev(TaskDevContent),
}

/// The content that is specific to each news type variant.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum TaskList {
    Dev(TaskListDevContent),
}
