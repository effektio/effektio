use anyhow::Result;
use futures::channel::mpsc::Sender;
use log::{info, warn};
use matrix_sdk::{
    encryption::identities::Device as MatrixDevice,
    ruma::{device_id, events::key::verification::VerificationMethod, OwnedUserId},
    Client,
};

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

    pub async fn get_unverified_devices(&self) -> Result<Vec<Device>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c
                    .user_id()
                    .expect("guest user cannot get the unverified devices");
                let mut devices: Vec<Device> = vec![];
                for dev in c.encryption().get_user_devices(user_id).await?.devices() {
                    if !dev.clone().verified() {
                        devices.push(Device::new(dev));
                    }
                }
                Ok(devices)
            })
            .await?
    }

    pub async fn request_verification(&self, dev_id: String) -> Result<bool> {
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

    pub async fn request_verification_with_methods(
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

    pub async fn get_deleted_devices(&self) -> Result<Vec<Device>> {
        let c = self.client.clone();
        RUNTIME
            .spawn(async move {
                let user_id = c
                    .user_id()
                    .expect("guest user cannot get the deleted devices");
                let mut devices: Vec<Device> = vec![];
                for dev in c.encryption().get_user_devices(user_id).await?.devices() {
                    if !dev.clone().deleted() {
                        devices.push(Device::new(dev));
                    }
                }
                Ok(devices)
            })
            .await?
    }
}

pub struct Device {
    inner: MatrixDevice,
}

impl Device {
    pub(crate) fn new(inner: MatrixDevice) -> Self {
        Self { inner }
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
}

pub fn handle_devices_changed_event(
    user_id: OwnedUserId,
    client: &Client,
    tx: &mut Sender<DevicesChangedEvent>,
) {
    info!("device-changed user_id: {}", user_id);
    let current_user_id = client
        .user_id()
        .expect("guest user cannot handle the device changed event");
    if user_id == *current_user_id {
        let evt = DevicesChangedEvent::new(client);
        if let Err(e) = tx.try_send(evt) {
            warn!("Dropping devices changed event: {}", e);
        }
    }
}

pub fn handle_devices_left_event(
    user_id: OwnedUserId,
    client: &Client,
    tx: &mut Sender<DevicesLeftEvent>,
) {
    info!("device-left user_id: {}", user_id);
    let current_user_id = client
        .user_id()
        .expect("guest user cannot handle the device left event");
    if user_id == *current_user_id {
        let evt = DevicesLeftEvent::new(client);
        if let Err(e) = tx.try_send(evt) {
            warn!("Dropping devices left event: {}", e);
        }
    }
}
