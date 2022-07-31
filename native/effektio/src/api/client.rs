use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use effektio_core::{
    models::{Faq, News},
    statics::{PURPOSE_FIELD, PURPOSE_FIELD_DEV, PURPOSE_TEAM_VALUE},
    RestoreToken,
};

#[cfg(feature = "with-mocks")]
use effektio_core::mocks::gen_mock_faqs;
use futures::{
    channel::mpsc::{channel, Receiver},
    stream, Stream, StreamExt,
};
use futures_signals::signal::{
    channel as signal_channel, Receiver as SignalReceiver, SignalExt, SignalStream,
};
use log::info;
use matrix_sdk::{
    config::SyncSettings,
    media::{MediaFormat, MediaRequest},
    ruma::{device_id, events::AnySyncRoomEvent, OwnedUserId, RoomId},
    Client as MatrixClient, LoopCtrl,
};
use parking_lot::{Mutex, RwLock};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::{
    api::FfiBuffer,
    events::{
        handle_devices_changed_event, handle_devices_left_event, handle_emoji_sync_msg_event,
        handle_emoji_to_device_event, DevicesChangedEvent, DevicesLeftEvent,
        EmojiVerificationEvent,
    },
    Account, Conversation, Group, Room, RUNTIME,
};

#[derive(Default, Builder, Debug)]
pub struct ClientState {
    #[builder(default)]
    pub is_guest: bool,
    #[builder(default)]
    pub has_first_synced: bool,
    #[builder(default)]
    pub is_syncing: bool,
    #[builder(default)]
    pub should_stop_syncing: bool,
}

#[derive(Clone)]
pub struct Client {
    pub(crate) client: MatrixClient,
    pub(crate) state: Arc<RwLock<ClientState>>,
}

impl std::ops::Deref for Client {
    type Target = MatrixClient;
    fn deref(&self) -> &MatrixClient {
        &self.client
    }
}

pub(crate) async fn devide_groups_from_common(
    client: MatrixClient,
) -> (Vec<Group>, Vec<Conversation>) {
    let (groups, convos, _) = stream::iter(client.clone().rooms().into_iter())
        .fold(
            (Vec::new(), Vec::new(), client),
            async move |(mut groups, mut conversations, client), room| {
                let is_effektio_group = {
                    #[allow(clippy::match_like_matches_macro)]
                    if let Ok(Some(_)) = room
                        .get_state_event(PURPOSE_FIELD.into(), PURPOSE_TEAM_VALUE)
                        .await
                    {
                        true
                    } else if let Ok(Some(_)) = room
                        .get_state_event(PURPOSE_FIELD_DEV.into(), PURPOSE_TEAM_VALUE)
                        .await
                    {
                        true
                    } else {
                        false
                    }
                };

                if is_effektio_group {
                    groups.push(Group {
                        inner: Room {
                            room,
                            client: client.clone(),
                        },
                    });
                } else {
                    conversations.push(Conversation {
                        inner: Room {
                            room,
                            client: client.clone(),
                        },
                    });
                }

                (groups, conversations, client)
            },
        )
        .await;
    (groups, convos)
}

#[derive(Clone)]
pub struct SyncState {
    emoji_verification_event_rx: Arc<Mutex<Option<Receiver<EmojiVerificationEvent>>>>, // mutex for sync, arc for clone. once called, it will become None, not Some
    devices_changed_event_rx: Arc<Mutex<Option<Receiver<DevicesChangedEvent>>>>, // mutex for sync, arc for clone. once called, it will become None, not Some
    devices_left_event_rx: Arc<Mutex<Option<Receiver<DevicesLeftEvent>>>>, // mutex for sync, arc for clone. once called, it will become None, not Some
    first_synced_rx: Arc<Mutex<Option<SignalReceiver<bool>>>>,
}

impl SyncState {
    pub fn new(
        emoji_verification_event_rx: Receiver<EmojiVerificationEvent>,
        devices_changed_event_rx: Receiver<DevicesChangedEvent>,
        devices_left_event_rx: Receiver<DevicesLeftEvent>,
        first_synced_rx: SignalReceiver<bool>,
    ) -> Self {
        let emoji_verification_event_rx = Arc::new(Mutex::new(Some(emoji_verification_event_rx)));
        let devices_changed_event_rx = Arc::new(Mutex::new(Some(devices_changed_event_rx)));
        let devices_left_event_rx = Arc::new(Mutex::new(Some(devices_left_event_rx)));
        let first_synced_rx = Arc::new(Mutex::new(Some(first_synced_rx)));

        Self {
            emoji_verification_event_rx,
            devices_changed_event_rx,
            devices_left_event_rx,
            first_synced_rx,
        }
    }

    pub fn get_emoji_verification_event_rx(&self) -> Option<Receiver<EmojiVerificationEvent>> {
        self.emoji_verification_event_rx.lock().take()
    }

    pub fn get_devices_changed_event_rx(&self) -> Option<Receiver<DevicesChangedEvent>> {
        self.devices_changed_event_rx.lock().take()
    }

    pub fn get_devices_left_event_rx(&self) -> Option<Receiver<DevicesLeftEvent>> {
        self.devices_left_event_rx.lock().take()
    }

    pub fn get_first_synced_rx(&self) -> Option<SignalStream<SignalReceiver<bool>>> {
        self.first_synced_rx.lock().take().map(|t| t.to_stream())
    }
}

impl Client {
    pub fn new(client: MatrixClient, state: ClientState) -> Self {
        Client {
            client,
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn start_sync(&self) -> SyncState {
        let client = self.client.clone();
        let state = self.state.clone();
        let (first_synced_tx, first_synced_rx) = futures_signals::signal::channel(false);

        let (emoji_verification_event_tx, emoji_verification_event_rx) =
            channel::<EmojiVerificationEvent>(10); // dropping after more than 10 items queued
        let (devices_changed_event_tx, devices_changed_event_rx) =
            channel::<DevicesChangedEvent>(10); // dropping after more than 10 items queued
        let (devices_left_event_tx, devices_left_event_rx) = channel::<DevicesLeftEvent>(10); // dropping after more than 10 items queued
        let emoji_verification_event_arc = Arc::new(emoji_verification_event_tx);
        let devices_changed_event_arc = Arc::new(devices_changed_event_tx);
        let devices_left_event_arc = Arc::new(devices_left_event_tx);
        let first_synced_arc = Arc::new(first_synced_tx);
        let initial_sync = Arc::new(AtomicBool::from(true));
        let sync_state = SyncState::new(
            emoji_verification_event_rx,
            devices_changed_event_rx,
            devices_left_event_rx,
            first_synced_rx,
        );

        RUNTIME.spawn(async move {
            let client = client.clone();
            let state = state.clone();

            client
                .clone()
                .sync_with_callback(SyncSettings::new(), move |response| {
                    let client = client.clone();
                    let state = state.clone();
                    let emoji_verification_event_arc = emoji_verification_event_arc.clone();
                    let devices_changed_event_arc = devices_changed_event_arc.clone();
                    let devices_left_event_arc = devices_left_event_arc.clone();
                    let initial_sync = initial_sync.clone();
                    let first_synced_arc = first_synced_arc.clone();

                    async move {
                        let client = client.clone();
                        let state = state.clone();
                        let initial = initial_sync.clone();
                        let mut emoji_verification_event_tx =
                            (*emoji_verification_event_arc).clone();
                        let mut devices_changed_event_tx = (*devices_changed_event_arc).clone();
                        let mut devices_left_event_tx = (*devices_left_event_arc).clone();

                        for user_id in response.device_lists.changed {
                            handle_devices_changed_event(
                                &user_id,
                                &client,
                                &mut devices_changed_event_tx,
                            );
                        }

                        for user_id in response.device_lists.left {
                            handle_devices_left_event(
                                &user_id,
                                &client,
                                &mut devices_left_event_tx,
                            );
                        }

                        for event in response.to_device.events {
                            if let Ok(evt) = event.deserialize() {
                                let json = serde_json::from_str::<Value>(event.json().get())
                                    .expect("Invalid JSON in to_device event");
                                info!("to_device event type: {}", json["type"]);
                                handle_emoji_to_device_event(
                                    &client,
                                    &evt,
                                    &mut emoji_verification_event_tx,
                                );
                            }
                        }

                        if !initial.load(Ordering::SeqCst) {
                            for (room_id, room_info) in response.rooms.join {
                                for event in room_info
                                    .timeline
                                    .events
                                    .iter()
                                    .filter_map(|ev| ev.event.deserialize().ok())
                                {
                                    if let AnySyncRoomEvent::MessageLike(evt) = event {
                                        handle_emoji_sync_msg_event(
                                            &client,
                                            &room_id,
                                            &evt,
                                            &mut emoji_verification_event_tx,
                                        );
                                    }
                                }
                            }
                        }

                        initial.store(false, Ordering::SeqCst);

                        let _ = first_synced_arc.send(true);
                        if !(*state).read().has_first_synced {
                            (*state).write().has_first_synced = true
                        }
                        if (*state).read().should_stop_syncing {
                            (*state).write().is_syncing = false;
                            // the lock is unlocked here when `s` goes out of scope.
                            return LoopCtrl::Break;
                        } else if !(*state).read().is_syncing {
                            (*state).write().is_syncing = true;
                        }
                        // the lock is unlocked here when `s` goes out of scope.
                        LoopCtrl::Continue
                    }
                })
                .await;
        });
        sync_state
    }

    /// Indication whether we've received a first sync response since
    /// establishing the client (in memory)
    pub fn has_first_synced(&self) -> bool {
        self.state.read().has_first_synced
    }

    /// Indication whether we are currently syncing
    pub fn is_syncing(&self) -> bool {
        self.state.read().is_syncing
    }

    /// Is this a guest account?
    pub fn is_guest(&self) -> bool {
        self.state.read().is_guest
    }

    pub async fn restore_token(&self) -> Result<String> {
        let session = self.client.session().context("Missing session")?.clone();
        let homeurl = self.client.homeserver().await;
        Ok(serde_json::to_string(&RestoreToken {
            session,
            homeurl,
            is_guest: self.state.read().is_guest,
        })?)
    }

    pub async fn conversations(&self) -> Result<Vec<Conversation>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let (_, conversations) = devide_groups_from_common(c).await;
                Ok(conversations)
            })
            .await?
    }

    #[cfg(feature = "with-mocks")]
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

    pub async fn user_id(&self) -> Result<OwnedUserId> {
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = l.user_id().context("No User ID found")?.to_owned();
                Ok(user_id)
            })
            .await?
    }

    pub async fn room(&self, room_name: String) -> Result<Room> {
        let room_id = RoomId::parse(room_name)?;
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                if let Some(room) = l.get_room(&room_id) {
                    return Ok(Room {
                        room,
                        client: l.clone(),
                    });
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
                let device_id = l.device_id().context("No Device ID found")?;
                Ok(device_id.as_str().to_string())
            })
            .await?
    }

    pub async fn avatar(&self) -> Result<FfiBuffer<u8>> {
        self.account().await?.avatar().await
    }

    pub async fn verified_device(&self, dev_id: String) -> Result<bool> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c.user_id().expect("guest user cannot request verification");
                let dev = c
                    .encryption()
                    .get_device(user_id, device_id!(dev_id.as_str()))
                    .await
                    .expect("alice should get device")
                    .unwrap();
                Ok(dev.verified())
            })
            .await?
    }
}
