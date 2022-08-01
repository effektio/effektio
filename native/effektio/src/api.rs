use anyhow::Result;
use futures::Stream;
use lazy_static::lazy_static;
pub use ruma;
use tokio::runtime;

use crate::{events, platform};

lazy_static! {
    pub static ref RUNTIME: runtime::Runtime =
        runtime::Runtime::new().expect("Can't start Tokio runtime");
}

mod account;
mod auth;
mod client;
mod conversation;
mod group;
mod messages;
mod news;
mod room;
mod stream;

pub use account::Account;
pub use auth::{
    guest_client, login_new_client, login_with_token, register_with_registration_token,
};
pub use client::{Client, ClientStateBuilder, SyncState};
pub use conversation::Conversation;
pub use effektio_core::models::{Color, Faq, News, Tag};
pub use events::{
    Device, DevicesChangedEvent, DevicesLeftEvent, EmojiUnit, EmojiVerificationEvent, TypingNotification,
};
pub use group::{CreateGroupSettings, CreateGroupSettingsBuilder, Group};
pub use messages::{FileDescription, ImageDescription, RoomMessage};
pub use room::{Member, Room};
pub use stream::TimelineStream;

#[cfg(feature = "with-mocks")]
pub use effektio_core::mocks::*;

pub type UserId = effektio_core::ruma::OwnedUserId;
pub type EventId = effektio_core::ruma::OwnedEventId;

#[cfg(all(not(doctest), feature = "dart"))]
ffi_gen_macro::ffi_gen!("native/effektio/api.rsh");

#[cfg(not(all(not(doctest), feature = "dart")))]
#[allow(clippy::module_inception)]
mod api {
    /// helpers for doctests, as ffigen for some reason can't find the path
    pub struct FfiBuffer<T>(Vec<T>);
    impl<T> FfiBuffer<T> {
        pub fn new(inner: Vec<T>) -> FfiBuffer<T> {
            FfiBuffer(inner)
        }
    }
}

fn init_logging(filter: Option<String>) -> Result<()> {
    platform::init_logging(filter)?;
    Ok(())
}
