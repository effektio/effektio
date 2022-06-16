use super::{api, Account, Conversation, Group, Room, RUNTIME};
use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use effektio_core::{
    mocks::{gen_mock_faqs, gen_mock_news},
    models::{Faq, News},
    ruma::api::client::account::register,
    RestoreToken,
};
use futures::{future::try_join_all, stream, Stream, StreamExt};
use lazy_static::lazy_static;
pub use matrix_sdk::ruma::{self, DeviceId, MxcUri, RoomId, ServerName};
use matrix_sdk::{
    media::{MediaFormat, MediaRequest},
    room::Room as MatrixRoom,
    ruma::events::StateEventType,
    Client as MatrixClient, LoopCtrl, Session,
};

use parking_lot::RwLock;
use ruma::events::room::MediaSource;
use std::sync::Arc;
use url::Url;

#[derive(Default, Builder, Debug)]
pub struct ClientState {
    #[builder(default)]
    pub is_guest: bool,
    #[builder(default)]
    pub has_first_synced: bool,
    #[builder(default)]
    pub is_syncing: bool,
    #[builder(default)]
    pub is_catching_up: bool,
    #[builder(default)]
    pub is_cought_up: bool,
    #[builder(default)]
    pub should_stop_syncing: bool,
}

#[derive(Clone)]
pub struct Client {
    client: MatrixClient,
    state: Arc<RwLock<ClientState>>,
}

impl std::ops::Deref for Client {
    type Target = MatrixClient;
    fn deref(&self) -> &MatrixClient {
        &self.client
    }
}

static PURPOSE_FIELD: &str = "m.room.purpose";
static PURPOSE_FIELD_DEV: &str = "org.matrix.msc3088.room.purpose";
static PURPOSE_VALUE: &str = "org.effektio";

// public API
impl Client {
    pub(crate) fn new(client: MatrixClient, state: ClientState) -> Self {
        Client {
            client,
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub(crate) fn start_sync(&self) {
        let me = self.clone();
        let client = self.client.clone();
        let state = self.state.clone();
        RUNTIME.spawn(async move {
            client
                .sync_with_callback(matrix_sdk::config::SyncSettings::new(), |_response| async {
                    if !state.read().has_first_synced {
                        state.write().has_first_synced = true;
                        me.catch_up();
                    }

                    if state.read().should_stop_syncing {
                        state.write().is_syncing = false;
                        return LoopCtrl::Break;
                    } else if !state.read().is_syncing {
                        state.write().is_syncing = true;
                    }
                    LoopCtrl::Continue
                })
                .await;
        });
    }

    /// Indication whether we've received a first sync response since
    /// establishing the client (in memory)
    pub fn has_first_synced(&self) -> bool {
        self.state.read().has_first_synced
    }

    /// Indication whether we are currently syncing
    pub fn is_syncing(&self) -> bool {
        self.state.read().has_first_synced
    }

    /// Is this a guest account?
    pub fn is_guest(&self) -> bool {
        self.state.read().is_guest
    }

    pub async fn restore_token(&self) -> Result<String> {
        let session = self.client.session().await.context("Missing session")?;
        let homeurl = self.client.homeserver().await;
        Ok(serde_json::to_string(&RestoreToken {
            session,
            homeurl,
            is_guest: self.state.read().is_guest,
        })?)
    }

    pub async fn conversations(&self) -> Result<Vec<Conversation>> {
        let me = self.clone();
        RUNTIME
            .spawn(async move { Ok(me.get_convos().await) })
            .await?
    }

    pub async fn groups(&self) -> Result<Vec<Group>> {
        let me = self.clone();
        RUNTIME
            .spawn(async move { Ok(me.get_groups().await) })
            .await?
    }

    pub async fn latest_news(&self) -> Result<Vec<News>> {
        Ok(gen_mock_news())
    }

    pub async fn faqs(&self) -> Result<Vec<Faq>> {
        Ok(gen_mock_faqs())
    }

    // pub async fn get_mxcuri_media(&self, uri: String) -> Result<Vec<u8>> {
    //     let l = self.client.clone();
    //     RUNTIME.spawn(async move {
    //         let user_id = l.user_id().await.expect("No User ID found");
    //         Ok(user_id.as_str().to_string())
    //     }).await?
    // }

    pub async fn user_id(&self) -> Result<ruma::OwnedUserId> {
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = l.user_id().await.context("No User ID found")?;
                Ok(user_id)
            })
            .await?
    }

    pub async fn room(&self, room_name: String) -> Result<Room> {
        let room_id = RoomId::parse(room_name)?;
        let l = self.client.clone();
        let client = self.clone();
        RUNTIME
            .spawn(async move {
                if let Some(room) = l.get_room(&room_id) {
                    return Ok(Room { room, client });
                }
                bail!("Room not found")
            })
            .await?
    }

    pub async fn account(&self) -> Result<Account> {
        Ok(Account::new(self.client.account()))
    }

    pub async fn display_name(&self) -> Result<String> {
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                let display_name = l
                    .account()
                    .get_display_name()
                    .await?
                    .context("No User ID found")?;
                Ok(display_name.as_str().to_string())
            })
            .await?
    }

    pub async fn device_id(&self) -> Result<String> {
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                let device_id = l.device_id().await.context("No Device ID found")?;
                Ok(device_id.as_str().to_string())
            })
            .await?
    }

    pub async fn avatar(&self) -> Result<api::FfiBuffer<u8>> {
        self.account().await?.avatar().await
    }
}

// Internal API
impl Client {
    async fn devide_groups_from_common(&self) -> (Vec<MatrixRoom>, Vec<MatrixRoom>) {
        stream::iter(self.client.rooms().into_iter())
            .fold(
                (Vec::new(), Vec::new()),
                async move |(mut groups, mut conversations), room| {
                    let is_effektio_group = {
                        #[allow(clippy::match_like_matches_macro)]
                        if let Ok(Some(_)) = room
                            .get_state_event(PURPOSE_FIELD.into(), PURPOSE_VALUE)
                            .await
                        {
                            true
                        } else if let Ok(Some(_)) = room
                            .get_state_event(PURPOSE_FIELD_DEV.into(), PURPOSE_VALUE)
                            .await
                        {
                            true
                        } else {
                            false
                        }
                    };

                    if is_effektio_group {
                        groups.push(room);
                    } else {
                        conversations.push(room);
                    }

                    (groups, conversations)
                },
            )
            .await
    }

    async fn get_groups(&self) -> Vec<Group> {
        let (groups, _) = self.devide_groups_from_common().await;
        groups
            .into_iter()
            .map(|room| Group {
                inner: Room {
                    room,
                    client: self.clone(),
                },
            })
            .collect()
    }
    async fn get_convos(&self) -> Vec<Conversation> {
        let (_, convos) = self.devide_groups_from_common().await;
        convos
            .into_iter()
            .map(|room| Conversation {
                inner: Room {
                    room,
                    client: self.clone(),
                },
            })
            .collect()
    }

    async fn catch_up(&self) -> Result<()> {
        let groups = self.get_groups().await;
        try_join_all(groups.iter().map(|g| g.sync_up())).await?;
        Ok(())
    }
}
