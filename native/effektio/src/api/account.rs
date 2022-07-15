use anyhow::{Context, Result};
pub use matrix_sdk::ruma::{self, DeviceId, MxcUri, RoomId, ServerName};
use matrix_sdk::{media::MediaFormat, Account as MatrixAccount};
use std::io::Cursor;
use url::Url;

use super::{api, RUNTIME};

#[derive(Clone, Debug)]
pub struct Account {
    account: MatrixAccount,
    user_id: String,
}

impl std::ops::Deref for Account {
    type Target = MatrixAccount;
    fn deref(&self) -> &MatrixAccount {
        &self.account
    }
}

impl Account {
    pub fn new(account: MatrixAccount, user_id: String) -> Self {
        Account { account, user_id }
    }

    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    pub async fn display_name(&self) -> Result<String> {
        let l = self.account.clone();
        RUNTIME
            .spawn(async move {
                let display_name = l.get_display_name().await?.context("No User ID found")?;
                Ok(display_name.as_str().to_string())
            })
            .await?
    }

    pub async fn set_display_name(&self, new_name: String) -> Result<bool> {
        let l = self.account.clone();
        RUNTIME
            .spawn(async move {
                let name = if new_name.is_empty() {
                    None
                } else {
                    Some(new_name.as_str())
                };
                l.set_display_name(name).await?;
                Ok(true)
            })
            .await?
    }

    pub async fn avatar(&self) -> Result<api::FfiBuffer<u8>> {
        let l = self.account.clone();
        RUNTIME
            .spawn(async move {
                let data = l
                    .get_avatar(MediaFormat::File)
                    .await?
                    .context("No avatar Url given")?;
                Ok(api::FfiBuffer::new(data))
            })
            .await?
    }

    pub async fn set_avatar(&self, c_type: String, data: Vec<u8>) -> Result<bool> {
        let l = self.account.clone();
        let mut c = Cursor::new(data);
        RUNTIME
            .spawn(async move {
                let new_url = l.upload_avatar(&c_type.parse()?, &mut c).await?;
                Ok(true)
            })
            .await?
    }
}
