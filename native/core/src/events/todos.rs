use super::TextMessageEventContent;
use derive_builder::Builder;
use matrix_sdk::ruma::{events::macros::EventContent, OwnedUserId};
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
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent, Builder)]
#[ruma_event(type = "org.effektio.dev.tasklist", kind = MessageLike)]
#[builder(name = "TaskListBuilder")]
pub struct TaskListContent {
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub role: Option<SpecialTaskListRole>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<TextMessageEventContent>,
    #[builder(setter(into, strip_option), default)]
    pub color: Option<Color>,
    #[builder(default)]
    pub sort_order: u32,
    #[builder(setter(into, strip_option), default)]
    pub time_zone: Option<TimeZone>,
    // FIXME: manage through `label` as in [MSC2326](https://github.com/matrix-org/matrix-doc/pull/2326)
    #[builder(setter(into, strip_option), default)]
    pub keywords: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub categories: Option<Vec<String>>,
}

/// The Task Event
///
/// modeled after [JMAP Task](https://jmap.io/spec-tasks.html#tasks)
/// see also the [IETF Task](https://www.rfc-editor.org/rfc/rfc8984.html#name-task)
/// but all timezones have been dumbed down to UTC-only.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent, Builder)]
#[ruma_event(type = "org.effektio.dev.task", kind = MessageLike)]
#[builder(name = "TaskBuilder")]
pub struct TaskContent {
    /// The title of the Task
    pub title: String,
    /// Every tasks belongs to a tasklist
    #[builder(setter(into))]
    #[serde(rename = "m.relates_to")]
    pub task_list_id: BelongsTo,
    /// Further information describing the task
    #[builder(setter(into, strip_option), default)]
    pub description: Option<TextMessageEventContent>,
    /// The users this task is assigned to
    #[builder(default)]
    pub assignees: Vec<OwnedUserId>,
    /// When is this task due
    #[builder(setter(into, strip_option), default)]
    pub utc_due: Option<UtcDateTime>,
    /// When was this task started?
    #[builder(setter(into, strip_option), default)]
    pub utc_start: Option<UtcDateTime>,
    /// How far along is this task in percent (everything > 100: = 100)
    #[builder(default)]
    pub progress_percent: Option<u8>,
    /// Sort order within the TaskList
    #[builder(default)]
    pub sort_order: u32,
    /// the priority of the Task
    #[builder(default)]
    pub priority: Option<Priority>,
    /// Color this task
    #[builder(setter(into, strip_option), default)]
    pub color: Option<Color>,
    // FIXME: manage through `label` as in [MSC2326](https://github.com/matrix-org/matrix-doc/pull/2326)
    #[builder(setter(into, strip_option), default)]
    pub keywords: Option<Vec<String>>,
    #[builder(setter(into, strip_option), default)]
    pub categories: Option<Vec<String>>,
}
