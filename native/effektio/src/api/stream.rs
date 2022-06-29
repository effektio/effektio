use anyhow::{Context, Result};
use core::pin::Pin;
use futures::{lock::Mutex, pin_mut, StreamExt};
use matrix_sdk::{
    deserialized_responses::SyncRoomEvent,
    room::Room as MatrixRoom,
    ruma::events::{AnySyncRoomEvent, AnySyncStateEvent},
    Client,
};
use std::sync::Arc;

use super::messages::{sync_event_to_message, RoomMessage};
use super::room::{sync_event_to_history, Invitation, Room};
use super::RUNTIME;

type BackwardEventStream =
    Pin<Box<dyn futures::Stream<Item = Result<SyncRoomEvent, matrix_sdk::Error>> + Send>>;
type ForwardEventStream = Pin<Box<dyn futures::Stream<Item = SyncRoomEvent> + Send>>;

// message event stream

#[derive(Clone)]
pub struct TimelineStream {
    client: Client,
    room: MatrixRoom,
    backward: Arc<Mutex<BackwardEventStream>>,
    forward: Arc<Mutex<ForwardEventStream>>,
}

unsafe impl Send for TimelineStream {}
unsafe impl Sync for TimelineStream {}

impl TimelineStream {
    pub fn new(
        forward: ForwardEventStream,
        backward: BackwardEventStream,
        client: Client,
        room: MatrixRoom,
    ) -> Self {
        TimelineStream {
            forward: Arc::new(Mutex::new(forward)),
            backward: Arc::new(Mutex::new(backward)),
            client,
            room,
        }
    }

    pub async fn paginate_backwards(&self, mut count: u64) -> Result<Vec<RoomMessage>> {
        let backward = self.backward.clone();
        let room = self.room.clone();
        RUNTIME
            .spawn(async move {
                let mut messages: Vec<RoomMessage> = Vec::new();
                let stream = backward.lock().await;
                pin_mut!(stream);

                while count > 0 {
                    match stream.next().await {
                        Some(Ok(e)) => {
                            if let Some(inner) = sync_event_to_message(e, room.clone()) {
                                messages.push(inner);
                                count -= 1;
                            }
                        }
                        None => {
                            // end of stream
                            break;
                        }
                        _ => {
                            // error cases, skipping
                        }
                    }
                }

                Ok(messages)
            })
            .await?
    }

    pub async fn next(&self) -> Result<RoomMessage> {
        let forward = self.forward.clone();
        let room = self.room.clone();
        RUNTIME
            .spawn(async move {
                let stream = forward.lock().await;
                pin_mut!(stream);
                loop {
                    if let Some(e) = stream
                        .next()
                        .await
                        .and_then(|e| sync_event_to_message(e, room.clone()))
                    {
                        return Ok(e);
                    }
                }
            })
            .await?
    }
}

// invitation event stream

#[derive(Clone)]
pub struct InvitationStream {
    client: Client,
    room: MatrixRoom,
    backward: Arc<Mutex<BackwardEventStream>>,
    forward: Arc<Mutex<ForwardEventStream>>,
}

unsafe impl Send for InvitationStream {}
unsafe impl Sync for InvitationStream {}

impl InvitationStream {
    pub fn new(
        forward: ForwardEventStream,
        backward: BackwardEventStream,
        client: Client,
        room: MatrixRoom,
    ) -> Self {
        InvitationStream {
            forward: Arc::new(Mutex::new(forward)),
            backward: Arc::new(Mutex::new(backward)),
            client,
            room,
        }
    }

    pub async fn paginate_backwards(&self, mut count: u64) -> Result<Vec<Invitation>> {
        let backward = self.backward.clone();
        let room = self.room.clone();
        RUNTIME
            .spawn(async move {
                let mut invitations: Vec<Invitation> = Vec::new();
                let stream = backward.lock().await;
                pin_mut!(stream);

                while count > 0 {
                    match stream.next().await {
                        Some(Ok(e)) => {
                            if let Some(inner) = sync_event_to_history(e, room.clone()) {
                                invitations.push(inner);
                                count -= 1;
                            }
                            println!("count: {}", count);
                        }
                        None => {
                            // end of stream
                            println!("456");
                            break;
                        }
                        _ => {
                            // error cases, skipping
                            println!("789");
                        }
                    }
                }

                Ok(invitations)
            })
            .await?
    }

    pub async fn next(&self) -> Result<Invitation> {
        let forward = self.forward.clone();
        let room = self.room.clone();
        RUNTIME
            .spawn(async move {
                let stream = forward.lock().await;
                pin_mut!(stream);
                loop {
                    if let Some(e) = stream
                        .next()
                        .await
                        .and_then(|e| sync_event_to_history(e, room.clone()))
                    {
                        return Ok(e);
                    }
                }
            })
            .await?
    }
}
