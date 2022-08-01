use anyhow::Result;
use futures::channel::mpsc::Sender;
use log::{info, warn};
use matrix_sdk::{
    config::SyncSettings,
    encryption::verification::{SasVerification, Verification},
    ruma::{
        events::{
            key::verification::VerificationMethod, room::message::MessageType,
            AnySyncMessageLikeEvent, AnyToDeviceEvent, SyncMessageLikeEvent,
        },
        OwnedRoomId, UserId,
    },
    Client,
};

use crate::RUNTIME;

#[derive(Clone, Debug)]
pub struct EmojiVerificationEvent {
    client: Client,
    event_name: String,
    txn_id: String,
    sender: String,
}

impl EmojiVerificationEvent {
    pub(crate) fn new(client: &Client, event_name: String, txn_id: String, sender: String) -> Self {
        Self {
            client: client.clone(),
            event_name,
            txn_id,
            sender,
        }
    }

    pub fn get_event_name(&self) -> String {
        self.event_name.clone()
    }

    pub fn get_sender(&self) -> String {
        self.sender.clone()
    }

    pub async fn accept_verification_request(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                let request = client
                    .encryption()
                    .get_verification_request(&sender, txn_id.as_str())
                    .await
                    .expect("Could not get request object");
                request
                    .accept()
                    .await
                    .expect("Can't accept verification request");
                Ok(true)
            })
            .await?
    }

    pub async fn cancel_verification_request(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                let request = client
                    .encryption()
                    .get_verification_request(&sender, txn_id.as_str())
                    .await
                    .expect("Could not get request object");
                request
                    .cancel()
                    .await
                    .expect("Can't cancel verification request");
                Ok(true)
            })
            .await?
    }

    pub async fn accept_verification_request_with_methods(
        &self,
        methods: &mut Vec<String>,
    ) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        let _methods: Vec<VerificationMethod> =
            (*methods).iter().map(|e| e.as_str().into()).collect();
        RUNTIME
            .spawn(async move {
                let request = client
                    .encryption()
                    .get_verification_request(&sender, txn_id.as_str())
                    .await
                    .expect("Could not get request object");
                request
                    .accept_with_methods(_methods)
                    .await
                    .expect("Can't accept verification request");
                Ok(true)
            })
            .await?
    }

    pub async fn start_sas_verification(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                let request = client
                    .encryption()
                    .get_verification_request(&sender, txn_id.as_str())
                    .await
                    .expect("Could not get request object");
                let sas_verification = request
                    .start_sas()
                    .await
                    .expect("Can't accept verification request");
                Ok(sas_verification.is_some())
            })
            .await?
    }

    pub async fn accept_sas_verification(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    sas.accept().await.unwrap();
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await?
    }

    pub async fn cancel_sas_verification(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    sas.cancel().await.unwrap();
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await?
    }

    pub async fn send_verification_key(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                client.sync_once(SyncSettings::default()).await?;
                Ok(true)
            })
            .await?
    }

    pub async fn get_verification_emoji(&self) -> Result<Vec<EmojiUnit>> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    if let Some(items) = sas.emoji() {
                        let sequence = items
                            .iter()
                            .map(|e| {
                                EmojiUnit::new(
                                    e.symbol.chars().collect::<Vec<_>>()[0] as u32,
                                    e.description.to_string(),
                                )
                            })
                            .collect::<Vec<_>>();
                        return Ok(sequence);
                    }
                }
                Ok(vec![])
            })
            .await?
    }

    pub async fn confirm_sas_verification(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    sas.confirm().await.unwrap();
                    Ok(sas.is_done())
                } else {
                    Ok(false)
                }
            })
            .await?
    }

    pub async fn mismatch_sas_verification(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    sas.mismatch().await.unwrap();
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await?
    }

    pub async fn cancel_verification_key(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    sas.cancel().await.unwrap();
                    Ok(true)
                } else {
                    Ok(false)
                }
            })
            .await?
    }

    pub async fn review_verification_mac(&self) -> Result<bool> {
        let client = self.client.clone();
        let sender = UserId::parse(self.sender.clone()).expect("Couldn't parse the user id");
        let txn_id = self.txn_id.clone();
        RUNTIME
            .spawn(async move {
                if let Some(Verification::SasV1(sas)) = client
                    .encryption()
                    .get_verification(&sender, txn_id.as_str())
                    .await
                {
                    Ok(sas.is_done())
                } else {
                    Ok(false)
                }
            })
            .await?
    }
}

#[derive(Clone, Debug)]
pub struct EmojiUnit {
    symbol: u32,
    description: String,
}

impl EmojiUnit {
    pub(crate) fn new(symbol: u32, description: String) -> Self {
        EmojiUnit {
            symbol,
            description,
        }
    }

    pub fn get_symbol(&self) -> u32 {
        self.symbol
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }
}

// thread callback must be global function, not member function
pub fn handle_emoji_sync_msg_event(
    client: &Client,
    room_id: &OwnedRoomId,
    event: &AnySyncMessageLikeEvent,
    tx: &mut Sender<EmojiVerificationEvent>,
) {
    match event {
        AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(m)) => {
            if let MessageType::VerificationRequest(_) = &m.content.msgtype {
                let dev_id = client.device_id().expect("guest user cannot get device id");
                info!("{} got m.room.message", dev_id.to_string());
                let sender = m.sender.to_string();
                let txn_id = m.event_id.to_string();
                let evt = EmojiVerificationEvent::new(
                    client,
                    "m.room.message".to_owned(),
                    txn_id.clone(),
                    sender,
                );
                if let Err(e) = tx.try_send(evt) {
                    warn!("Dropping event for {}: {}", txn_id, e);
                }
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationReady(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.ready", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.ready".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationStart(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.start", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.start".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationCancel(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.cancel", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.cancel".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationAccept(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.accept", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.accept".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationKey(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.key", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.key".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationMac(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.mac", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.mac".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        AnySyncMessageLikeEvent::KeyVerificationDone(SyncMessageLikeEvent::Original(ev)) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.done", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.relates_to.event_id.as_str().to_owned();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.done".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping event for {}: {}", txn_id, e);
            }
        }
        _ => {}
    }
}

// thread callback must be global function, not member function
pub fn handle_emoji_to_device_event(
    client: &Client,
    event: &AnyToDeviceEvent,
    tx: &mut Sender<EmojiVerificationEvent>,
) {
    match event {
        AnyToDeviceEvent::KeyVerificationRequest(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.request", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.request".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationReady(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.ready", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.ready".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationStart(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.start", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.start".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationCancel(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.cancel", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.cancel".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationAccept(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.accept", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.accept".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationKey(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.key", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.key".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationMac(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.mac", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.mac".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        AnyToDeviceEvent::KeyVerificationDone(ev) => {
            let dev_id = client.device_id().expect("guest user cannot get device id");
            info!("{} got m.key.verification.done", dev_id.to_string());
            let sender = ev.sender.to_string();
            let txn_id = ev.content.transaction_id.to_string();
            let evt = EmojiVerificationEvent::new(
                client,
                "m.key.verification.done".to_owned(),
                txn_id.clone(),
                sender,
            );
            if let Err(e) = tx.try_send(evt) {
                warn!("Dropping transaction for {}: {}", txn_id, e);
            }
        }
        _ => {}
    }
}
