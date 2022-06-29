use anyhow::Result;
use futures::Stream;
use lazy_static::lazy_static;
pub use ruma;
use tokio::runtime;

use crate::platform;

lazy_static! {
    static ref RUNTIME: runtime::Runtime =
        runtime::Runtime::new().expect("Can't start Tokio runtime");
}

mod account;
mod auth;
mod client;
mod conversation;
mod group;
mod messages;
mod room;
mod stream;

pub use account::Account;
pub use auth::{
    guest_client, login_new_client, login_with_token, register_with_registration_token,
};
pub use client::{Client, ClientStateBuilder};
pub use conversation::Conversation;
pub use effektio_core::models::{Color, Faq, News, Tag};
pub use group::Group;
pub use messages::{FileDescription, ImageDescription, RoomMessage};
pub use room::{Invitation, Member, Room};
pub use stream::{InvitationStream, TimelineStream};

#[cfg(feature = "with-mocks")]
pub use effektio_core::mocks::*;

pub type UserId = ruma::OwnedUserId;
pub type EventId = ruma::OwnedEventId;

ffi_gen_macro::ffi_gen!("native/effektio/api.rsh");

fn init_logging(filter: Option<String>) -> Result<()> {
    platform::init_logging(filter)?;
    Ok(())
}
