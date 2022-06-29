use anyhow::{bail, Context, Result};
use futures::{pin_mut, stream, Stream, StreamExt};
use matrix_sdk::{
    attachment::{AttachmentConfig, AttachmentInfo, BaseFileInfo, BaseImageInfo},
    deserialized_responses::SyncRoomEvent,
    media::{MediaFormat, MediaRequest},
    room::Room as MatrixRoom,
    ruma::{
        events::{
            room::{
                member::{MembershipState, RoomMemberEventContent},
                message::{MessageType, RoomMessageEventContent},
            },
            AnyMessageLikeEvent, AnyMessageLikeEventContent, AnyRoomEvent, AnyStrippedStateEvent,
            AnySyncRoomEvent, AnySyncStateEvent, MessageLikeEvent, OriginalSyncStateEvent, StateEventType, SyncStateEvent,
        },
        EventId, OwnedUserId, UInt,
    },
    Client as MatrixClient,
};
use std::{fs::File, io::Write, path::PathBuf};

use super::messages::{sync_event_to_message, RoomMessage};
use super::{api, InvitationStream, TimelineStream, RUNTIME};

pub struct Member {
    pub(crate) member: matrix_sdk::RoomMember,
}

impl std::ops::Deref for Member {
    type Target = matrix_sdk::RoomMember;
    fn deref(&self) -> &matrix_sdk::RoomMember {
        &self.member
    }
}

impl Member {
    pub async fn avatar(&self) -> Result<api::FfiBuffer<u8>> {
        let r = self.member.clone();
        RUNTIME
            .spawn(async move {
                Ok(api::FfiBuffer::new(
                    r.avatar(MediaFormat::File).await?.context("No avatar")?,
                ))
            })
            .await?
    }
    pub fn display_name(&self) -> Option<String> {
        self.member.display_name().map(|s| s.to_owned())
    }

    pub fn user_id(&self) -> OwnedUserId {
        self.member.user_id().to_owned()
    }
}

pub struct Room {
    pub(crate) client: MatrixClient,
    pub(crate) room: MatrixRoom,
}

impl Room {
    pub async fn display_name(&self) -> Result<String> {
        let r = self.room.clone();
        RUNTIME
            .spawn(async move { Ok(r.display_name().await?.to_string()) })
            .await?
    }

    pub async fn avatar(&self) -> Result<api::FfiBuffer<u8>> {
        let r = self.room.clone();
        RUNTIME
            .spawn(async move {
                Ok(api::FfiBuffer::new(
                    r.avatar(MediaFormat::File).await?.context("No avatar")?,
                ))
            })
            .await?
    }

    pub async fn active_members(&self) -> Result<Vec<Member>> {
        let r = self.room.clone();
        RUNTIME
            .spawn(async move {
                Ok(r.active_members()
                    .await
                    .context("No members")?
                    .into_iter()
                    .map(|member| Member { member })
                    .collect())
            })
            .await?
    }

    pub async fn active_members_no_sync(&self) -> Result<Vec<Member>> {
        let r = self.room.clone();
        RUNTIME
            .spawn(async move {
                Ok(r.active_members_no_sync()
                    .await
                    .context("No members")?
                    .into_iter()
                    .map(|member| Member { member })
                    .collect())
            })
            .await?
    }

    pub async fn get_member(&self, user_id: Box<OwnedUserId>) -> Result<Member> {
        let r = self.room.clone();
        RUNTIME
            .spawn(async move {
                let member = r.get_member(&user_id).await?.context("User not found")?;
                Ok(Member { member })
            })
            .await?
    }

    pub async fn timeline(&self) -> Result<TimelineStream> {
        let room = self.room.clone();
        let client = self.client.clone();
        RUNTIME
            .spawn(async move {
                let (forward, backward) = room
                    .timeline()
                    .await
                    .context("Failed acquiring timeline streams")?;
                Ok(TimelineStream::new(
                    Box::pin(forward),
                    Box::pin(backward),
                    client,
                    room,
                ))
            })
            .await?
    }

    pub async fn invitation_history(&self) -> Result<InvitationStream> {
        let room = self.room.clone();
        let client = self.client.clone();
        RUNTIME
            .spawn(async move {
                let (forward, backward) = room
                    .timeline()
                    .await
                    .context("Failed acquiring invitation streams")?;
                Ok(InvitationStream::new(
                    Box::pin(forward),
                    Box::pin(backward),
                    client,
                    room,
                ))
            })
            .await?
    }

    pub async fn latest_message(&self) -> Result<RoomMessage> {
        let room = self.room.clone();
        RUNTIME
            .spawn(async move {
                let stream = room
                    .timeline_backward()
                    .await
                    .context("Failed acquiring timeline streams")?;
                pin_mut!(stream);
                loop {
                    match stream.next().await {
                        None => break,
                        Some(Ok(e)) => {
                            if let Some(a) = sync_event_to_message(e, room.clone()) {
                                return Ok(a);
                            }
                        }
                        _ => {
                            // we ignore errors
                        }
                    }
                }

                bail!("No Message found")
            })
            .await?
    }

    pub async fn typing_notice(&self, typing: bool) -> Result<bool> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send typing notice to a room we are not in")
        };
        RUNTIME
            .spawn(async move {
                room.typing_notice(typing).await?;
                Ok(true)
            })
            .await?
    }

    pub async fn read_receipt(&self, event_id: String) -> Result<bool> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send read_receipt to a room we are not in")
        };
        let event_id = EventId::parse(event_id)?;
        RUNTIME
            .spawn(async move {
                room.read_receipt(&event_id).await?;
                Ok(true)
            })
            .await?
    }

    pub async fn send_plain_message(&self, message: String) -> Result<String> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        RUNTIME
            .spawn(async move {
                let r = room
                    .send(
                        AnyMessageLikeEventContent::RoomMessage(
                            RoomMessageEventContent::text_plain(message),
                        ),
                        None,
                    )
                    .await?;
                Ok(r.event_id.to_string())
            })
            .await?
    }

    pub async fn send_image_message(
        &self,
        uri: String,
        name: String,
        mimetype: String,
        size: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<String> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        RUNTIME
            .spawn(async move {
                let path = PathBuf::from(uri);
                let mut image = File::open(path)?;
                let config = AttachmentConfig::new().info(AttachmentInfo::Image(BaseImageInfo {
                    height: height.map(UInt::from),
                    width: width.map(UInt::from),
                    size: size.map(UInt::from),
                    blurhash: None,
                }));
                let mime_type: mime::Mime = mimetype.parse().unwrap();
                let r = room
                    .send_attachment(name.as_str(), &mime_type, &mut image, config)
                    .await?;
                Ok(r.event_id.to_string())
            })
            .await?
    }

    pub async fn image_binary(&self, event_id: String) -> Result<api::FfiBuffer<u8>> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        let client = self.client.clone();
        // any variable in self can't be called directly in spawn
        RUNTIME
            .spawn(async move {
                let eid = EventId::parse(event_id.clone())?;
                let evt = room.event(&eid).await?;
                if let Ok(AnyRoomEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                    MessageLikeEvent::Original(m),
                ))) = evt.event.deserialize()
                {
                    if let MessageType::Image(content) = &m.content.msgtype {
                        let source = content.source.clone();
                        let data = client
                            .get_media_content(
                                &MediaRequest {
                                    source,
                                    format: MediaFormat::File,
                                },
                                false,
                            )
                            .await?;
                        Ok(api::FfiBuffer::new(data))
                    } else {
                        bail!("Invalid file format")
                    }
                } else {
                    bail!("Invalid file format")
                }
            })
            .await?
    }

    pub async fn send_file_message(
        &self,
        uri: String,
        name: String,
        mimetype: String,
        size: u32,
    ) -> Result<String> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        RUNTIME
            .spawn(async move {
                let path = PathBuf::from(uri);
                let mut image = File::open(path)?;
                let config = AttachmentConfig::new().info(AttachmentInfo::File(BaseFileInfo {
                    size: Some(UInt::from(size)),
                }));
                let mime_type: mime::Mime = mimetype.parse().unwrap();
                let r = room
                    .send_attachment(name.as_str(), &mime_type, &mut image, config)
                    .await?;
                Ok(r.event_id.to_string())
            })
            .await?
    }

    pub async fn save_file(&self, event_id: String, dir_path: String) -> Result<String> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        let client = self.client.clone();
        // any variable in self can't be called directly in spawn
        RUNTIME
            .spawn(async move {
                let eid = EventId::parse(event_id.clone())?;
                let evt = room.event(&eid).await?;
                if let Ok(AnyRoomEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                    MessageLikeEvent::Original(m),
                ))) = evt.event.deserialize()
                {
                    if let MessageType::File(content) = m.content.msgtype {
                        let source = content.source.clone();
                        let name = content.body.clone();
                        let mut path = PathBuf::from(dir_path.clone());
                        path.push(name);
                        let mut file = File::create(path.clone())?;
                        let data = client
                            .get_media_content(
                                &MediaRequest {
                                    source,
                                    format: MediaFormat::File,
                                },
                                false,
                            )
                            .await?;
                        file.write_all(&data)?;
                        let key =
                            [room.room_id().as_str().as_bytes(), event_id.as_bytes()].concat();
                        let path_text = path
                            .to_str()
                            .expect("Path was generated from strings. Must be string");
                        client
                            .store()
                            .set_custom_value(&key, path_text.as_bytes().to_vec())
                            .await?;
                        Ok(path_text.to_owned())
                    } else {
                        bail!("This message type is not file")
                    }
                } else {
                    bail!("It is not message")
                }
            })
            .await?
    }

    pub async fn file_path(&self, event_id: String) -> Result<String> {
        let room = if let MatrixRoom::Joined(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        let client = self.client.clone();
        RUNTIME
            .spawn(async move {
                let eid = EventId::parse(event_id.clone())?;
                let evt = room.event(&eid).await?;
                if let Ok(AnyRoomEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                    MessageLikeEvent::Original(m),
                ))) = evt.event.deserialize()
                {
                    if let MessageType::File(content) = m.content.msgtype {
                        let key = [
                            room.room_id().as_str().as_bytes(),
                            event_id.as_str().as_bytes(),
                        ]
                        .concat();
                        let path = client.store().get_custom_value(&key).await?;
                        if let Some(value) = path {
                            let text = std::str::from_utf8(&value).unwrap();
                            Ok(text.to_owned())
                        } else {
                            bail!("Couldn't get the path of saved file")
                        }
                    } else {
                        bail!("This message type is not file")
                    }
                } else {
                    bail!("It is not message")
                }
            })
            .await?
    }

    pub async fn get_inviter(&self) -> Result<String> {
        let room = if let MatrixRoom::Invited(r) = &self.room {
            r.clone()
        } else {
            bail!("Can't send message to a room we are not in")
        };
        let client = self.client.clone();
        RUNTIME
            .spawn(async move {
                let events = room.get_state_events(StateEventType::RoomMember).await?;
                println!("state events: {}", events.len());
                for event in events {
                    println!("xxx");
                    if let Ok(AnySyncStateEvent::RoomMember(member)) = event.deserialize() {
                        println!("sender: {}", member.sender());
                    }
                }
                Ok("123".to_owned())
            })
            .await?
    }
}

impl std::ops::Deref for Room {
    type Target = MatrixRoom;
    fn deref(&self) -> &MatrixRoom {
        &self.room
    }
}

pub struct Invitation {
    event_id: String,
    sender: String,
}

impl Invitation {
    pub fn get_event_id(&self) -> String {
        self.event_id.clone()
    }

    pub fn get_sender(&self) -> String {
        self.sender.clone()
    }
}

pub fn sync_event_to_history(sync_event: SyncRoomEvent, room: MatrixRoom) -> Option<Invitation> {
    if let Ok(AnySyncRoomEvent::State(state)) = sync_event.event.deserialize()
    {
        println!("state");
        println!("{:?}", state);
    }
    if let Ok(AnySyncRoomEvent::State(AnySyncStateEvent::RoomMember(member))) = sync_event.event.deserialize()
    {
        println!("sync_event");
        println!("{:?}", sync_event);
        println!("member");
        println!("{:?}", member.state_key());
        if let SyncStateEvent::Original(
            OriginalSyncStateEvent {
                content: RoomMemberEventContent {
                    avatar_url, displayname, is_direct, membership, third_party_invite, blurhash, reason, join_authorized_via_users_server, ..
                },
                event_id, sender, origin_server_ts, state_key, ..
            },
        ) = member
        {
            println!("123");
            if membership == MembershipState::Invite {
                return Some(Invitation {
                    event_id: event_id.to_string(),
                    sender: sender.to_string(),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use matrix_sdk::{
        config::SyncSettings,
        room::Room as MatrixRoom,
        ruma::{room_id, events::StateEventType},
        Client as MatrixClient, LoopCtrl,
    };
    use tokio::time::{Duration, sleep};
    use zenv::Zenv;

    use crate::api::{room::Room, Client, ClientStateBuilder, InvitationStream, login_new_client};

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

        let sync_settings = SyncSettings::new()/*.timeout(Duration::from_secs(5))*/;
        client.sync_once(sync_settings).await.unwrap();

        let room_id = room_id!("!jXsqlnitogAbTTSksT:effektio.org");
        let room = client.get_invited_room(room_id).unwrap();

        let room = Room {
            client: client.clone(),
            room: MatrixRoom::Invited(room),
        };
        let stream: InvitationStream = room.invitation_history().await?;
        let invitations = stream.paginate_backwards(10).await?;
        println!("invitation len: {}", invitations.len());

        // let settings = SyncSettings::default().token(client.sync_token().await.unwrap());
        // client.sync(settings).await;
        // println!("456");

        let c = Client::new(
            client,
            ClientStateBuilder::default().is_guest(false).build()?,
        );
        Ok(c)
    }

    #[tokio::test]
    async fn test_get_inviter() -> Result<()> {
        let z = Zenv::new(".env", false).parse()?;
        let homeserver_url: String = z.get("HOMESERVER_URL").unwrap().to_owned();
        let base_path: String = z.get("BASE_PATH").unwrap().to_owned();
        let username: String = z.get("USERNAME").unwrap().to_owned();
        let password: String = z.get("PASSWORD").unwrap().to_owned();

        // let client = login_and_sync(homeserver_url, base_path, username, password).await?;

        let client = login_new_client(base_path, username, password).await?;

        sleep(Duration::from_secs(5)).await;

        let room_id: String = "!jXsqlnitogAbTTSksT:effektio.org".to_owned();
        let room: Room = client.room(room_id).await.expect("Expected room to be available");
        let stream: InvitationStream = room.invitation_history().await?;
        let invitations = stream.paginate_backwards(3).await?;
        println!("invitation len: {}", invitations.len());

        assert_eq!(1, 1);
        Ok(())
    }
}
