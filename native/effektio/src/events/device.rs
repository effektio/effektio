use anyhow::Result;
use futures::channel::mpsc::Sender;
use log::{info, warn};
use matrix_sdk::{
    encryption::identities::Device as MatrixDevice,
    ruma::{
        device_id, events::key::verification::VerificationMethod, MilliSecondsSinceUnixEpoch,
        OwnedUserId,
    },
    Client,
};
use serde_json::{json, Value};

use crate::RUNTIME;

#[derive(Clone, Debug)]
pub struct DevicesChangedEvent {
    client: Client,
}

impl DevicesChangedEvent {
    pub(crate) fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
        }
    }

    pub async fn get_devices(&self, verified: bool) -> Result<Vec<Device>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c
                    .user_id()
                    .expect("guest user cannot get the verified devices");
                let mut devices: Vec<Device> = vec![];
                let response = c.devices().await?;
                for device in c.encryption().get_user_devices(user_id).await?.devices() {
                    if device.verified() == verified {
                        if let Some(dev) = response
                            .devices
                            .iter()
                            .find(|e| e.device_id == device.device_id())
                        {
                            devices.push(Device::new(
                                &device,
                                dev.last_seen_ip.clone(),
                                dev.last_seen_ts,
                            ));
                        } else {
                            devices.push(Device::new(&device, None, None));
                        }
                    }
                }
                Ok(devices)
            })
            .await?
    }

    pub async fn request_verification_to_user(&self) -> Result<bool> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c.user_id().expect("guest user cannot request verification");
                let user = c
                    .encryption()
                    .get_user_identity(user_id)
                    .await?
                    .expect("alice should get user identity");
                user.request_verification().await?;
                Ok(true)
            })
            .await?
    }

    pub async fn request_verification_to_device(&self, dev_id: String) -> Result<bool> {
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
                dev.request_verification_with_methods(vec![VerificationMethod::SasV1])
                    .await?;
                Ok(true)
            })
            .await?
    }

    pub async fn request_verification_to_user_with_methods(
        &self,
        methods: &mut Vec<String>,
    ) -> Result<bool> {
        let c = self.client.clone();
        let _methods: Vec<VerificationMethod> =
            (*methods).iter().map(|e| e.as_str().into()).collect();
        RUNTIME
            .spawn(async move {
                let user_id = c.user_id().expect("guest user cannot request verification");
                let user = c
                    .encryption()
                    .get_user_identity(user_id)
                    .await?
                    .expect("alice should get user identity");
                user.request_verification_with_methods(_methods).await?;
                Ok(true)
            })
            .await?
    }

    pub async fn request_verification_to_device_with_methods(
        &self,
        dev_id: String,
        methods: &mut Vec<String>,
    ) -> Result<bool> {
        let c = self.client.clone();
        let _methods: Vec<VerificationMethod> =
            (*methods).iter().map(|e| e.as_str().into()).collect();
        RUNTIME
            .spawn(async move {
                let user_id = c.user_id().expect("guest user cannot request verification");
                let dev = c
                    .encryption()
                    .get_device(user_id, device_id!(dev_id.as_str()))
                    .await
                    .expect("alice should get device")
                    .unwrap();
                dev.request_verification_with_methods(_methods).await?;
                Ok(true)
            })
            .await?
    }
}

#[derive(Clone, Debug)]
pub struct DevicesLeftEvent {
    client: Client,
}

impl DevicesLeftEvent {
    pub(crate) fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
        }
    }

    pub async fn get_devices(&self, deleted: bool) -> Result<Vec<Device>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c
                    .user_id()
                    .expect("guest user cannot get the deleted devices");
                let mut devices: Vec<Device> = vec![];
                let response = c.devices().await?;
                for device in c.encryption().get_user_devices(user_id).await?.devices() {
                    if device.deleted() == deleted {
                        if let Some(dev) = response
                            .devices
                            .iter()
                            .find(|e| e.device_id == device.device_id())
                        {
                            devices.push(Device::new(
                                &device,
                                dev.last_seen_ip.clone(),
                                dev.last_seen_ts,
                            ));
                        } else {
                            devices.push(Device::new(&device, None, None));
                        }
                    }
                }
                Ok(devices)
            })
            .await?
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    inner: MatrixDevice,
    last_seen_ip: Option<String>,
    last_seen_ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl Device {
    pub(crate) fn new(
        inner: &MatrixDevice,
        last_seen_ip: Option<String>,
        last_seen_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self {
            inner: inner.clone(),
            last_seen_ip,
            last_seen_ts,
        }
    }

    pub fn was_verified(&self) -> bool {
        self.inner.verified()
    }

    pub fn was_deleted(&self) -> bool {
        self.inner.deleted()
    }

    pub fn get_user_id(&self) -> String {
        self.inner.user_id().to_string()
    }

    pub fn get_device_id(&self) -> String {
        self.inner.device_id().to_string()
    }

    pub fn get_display_name(&self) -> Option<String> {
        self.inner.display_name().map(|s| s.to_owned())
    }

    pub fn get_last_seen_ip(&self) -> Option<String> {
        self.last_seen_ip.clone()
    }

    pub fn get_last_seen_ts(&self) -> Option<MilliSecondsSinceUnixEpoch> {
        self.last_seen_ts
    }
}

pub fn handle_devices_changed_event(
    user_id: &OwnedUserId,
    client: &Client,
    tx: &mut Sender<DevicesChangedEvent>,
) {
    info!("device-changed user_id: {}", user_id);
    let current_user_id = client.user_id().expect("guest user cannot get user id");
    let current_device_id = client.device_id().expect("guest user cannot get device id");
    if *user_id == *current_user_id {
        let evt = DevicesChangedEvent::new(client);
        if let Err(e) = tx.try_send(evt) {
            warn!("Dropping devices changed event: {}", e);
        }
    }
}

pub fn handle_devices_left_event(
    user_id: &OwnedUserId,
    client: &Client,
    tx: &mut Sender<DevicesLeftEvent>,
) {
    info!("device-left user_id: {}", user_id);
    let current_user_id = client
        .user_id()
        .expect("guest user cannot handle the device left event");
    if *user_id == *current_user_id {
        let evt = DevicesLeftEvent::new(client);
        if let Err(e) = tx.try_send(evt) {
            warn!("Dropping devices left event: {}", e);
        }
    }
}
