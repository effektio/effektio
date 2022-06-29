use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use effektio_core::{
    mocks::{gen_mock_faqs, gen_mock_news},
    models::{Faq, News},
    RestoreToken,
};
use futures::{stream, Stream, StreamExt};
use matrix_sdk::{
    config::SyncSettings,
    ruma::{
        events::{AnySyncStateEvent, StateEventType},
        DeviceId, MxcUri, OwnedUserId, RoomId, ServerName,
    },
    Client as MatrixClient, LoopCtrl,
};
use parking_lot::RwLock;
use std::sync::Arc;

use super::{api, Account, Conversation, Group, Room, RUNTIME};

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

async fn devide_groups_from_common(client: MatrixClient) -> (Vec<Group>, Vec<Conversation>) {
    let (groups, convos, _) = stream::iter(client.clone().rooms().into_iter())
        .fold(
            (Vec::new(), Vec::new(), client),
            async move |(mut groups, mut conversations, client), room| {
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

impl Client {
    pub(crate) fn new(client: MatrixClient, state: ClientState) -> Self {
        Client {
            client,
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub(crate) fn start_sync(&self) {
        let client = self.client.clone();
        let state = self.state.clone();
        RUNTIME.spawn(async move {
            client
                .sync_with_callback(SyncSettings::new(), |_response| async {
                    if !state.read().has_first_synced {
                        state.write().has_first_synced = true
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
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let (_, conversations) = devide_groups_from_common(c).await;
                Ok(conversations)
            })
            .await?
    }

    pub async fn groups(&self) -> Result<Vec<Group>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let (groups, _) = devide_groups_from_common(c).await;
                Ok(groups)
            })
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

    pub async fn user_id(&self) -> Result<OwnedUserId> {
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
                let device_id = l.device_id().await.context("No Device ID found")?;
                Ok(device_id.as_str().to_string())
            })
            .await?
    }

    pub async fn avatar(&self) -> Result<api::FfiBuffer<u8>> {
        self.account().await?.avatar().await
    }

    pub async fn get_inviter(&self, room_id: String) -> Result<String> {
        let room_id = RoomId::parse(room_id)?;
        let l = self.client.clone();
        RUNTIME
            .spawn(async move {
                if let Some(room) = l.get_invited_room(&room_id) {
                    let events = room.get_state_events(StateEventType::RoomMember).await?;
                    println!("state events: {}", events.len());
                    for event in events {
                        println!("xxx");
                        if let Ok(AnySyncStateEvent::RoomMember(member)) = event.deserialize() {
                            println!("sender: {}", member.sender());
                        }
                    }
                    return Ok("123".to_owned());
                }
                bail!("Room not found")
            })
            .await?
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use matrix_sdk::{
        config::SyncSettings,
        Client as MatrixClient, LoopCtrl,
    };
    use tokio::time::{Duration, sleep};
    use zenv::Zenv;

    use crate::api::{Client, ClientStateBuilder, login_new_client};

    async fn login_and_sync(
        homeserver_url: String,
        base_path: String,
        username: String,
        password: String,
    ) -> Result<Client> {
        let mut client_builder = MatrixClient::builder().homeserver_url(homeserver_url);

        #[cfg(feature = "sled")]
        {
            let state_store = matrix_sdk_sled::StateStore::open_with_path(base_path)?;
            client_builder = client_builder.state_store(state_store);
        }

        let client = client_builder.build().await.unwrap();
        client.login(&username, &password, None, Some("command bot")).await?;
        println!("logged in as {}", username);

        let sync_settings = SyncSettings::new().timeout(Duration::from_secs(5));
        client.sync_once(sync_settings).await.unwrap();

        // let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
        // client.sync(settings).await;
        // println!("456");

        let c = Client::new(
            client,
            ClientStateBuilder::default().is_guest(false).build()?,
        );
        Ok(c)
    }

    // #[tokio::test]
    async fn test_get_inviter() -> Result<()> {
        let z = Zenv::new(".env", false).parse()?;
        let homeserver_url: String = z.get("HOMESERVER_URL").unwrap().to_owned();
        let base_path: String = z.get("BASE_PATH").unwrap().to_owned();
        let username: String = z.get("USERNAME").unwrap().to_owned();
        let password: String = z.get("PASSWORD").unwrap().to_owned();

        let client = login_and_sync(homeserver_url, base_path, username, password).await?;

        // let client = login_new_client(base_path, username, password).await?;

        sleep(Duration::from_secs(5)).await;

        let room_id: String = "!jXsqlnitogAbTTSksT:effektio.org".to_owned();
        let res: String = client.get_inviter(room_id).await?;
        println!("inviter: {}", res);

        assert_eq!(1, 1);
        Ok(())
    }
}
