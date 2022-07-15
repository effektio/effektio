use anyhow::Result;
use matrix_sdk::{
    config::SyncSettings,
    ruma::events::AnyStrippedStateEvent,
    Client as MatrixClient, LoopCtrl,
};
use serde_json::Value;
use std::{option_env, time::Duration};
use tempfile::TempDir;
use tokio::time::sleep;

use effektio::api::{Client, ClientStateBuilder, login_new_client};

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
    client.login(&username.clone(), &password, None, Some("command bot")).await?;
    println!("logged in as {}", username);

    let sync_settings = SyncSettings::new().timeout(Duration::from_secs(10));
    client
        .sync_with_callback(sync_settings, move |response| {
            let username = username.clone();

            async move {
                for (room_id, room) in response.rooms.invite {
                    for event in room.invite_state.events {
                        if let Ok(AnyStrippedStateEvent::RoomMember(member)) = event.deserialize() {
                            if member.state_key == username {
                                println!("event: {:?}", event);
                                println!("member: {:?}", member);
                                let v: Value = serde_json::from_str(event.json().get()).unwrap();
                                println!("event id: {}", v["event_id"]);
                                println!("timestamp: {}", v["origin_server_ts"]);
                                println!("room id: {:?}", room_id);
                                println!("sender: {:?}", member.sender);
                                println!("state key: {:?}", member.state_key);
                            }
                        }
                    }
                }
                return LoopCtrl::Break;
            }
        })
        .await;

    let c = Client::new(
        client,
        ClientStateBuilder::default().is_guest(false).build()?,
    );
    Ok(c)
}

#[tokio::test]
async fn load_pending_invitation() -> Result<()> {
    let homeserver_url: String = option_env!("HOMESERVER")
        .unwrap_or("http://localhost:8008")
        .to_string();
    let tmp_dir = TempDir::new()?;
    let base_path: String = tmp_dir.path().to_str().expect("always works").to_owned();
    let username: String = "@sisko:ds9.effektio.org".to_owned();
    let password: String = "sisko".to_owned();

    // let client = login_and_sync(homeserver_url, base_path, username, password).await?;

    let client = login_new_client(base_path, username, password).await?;
    println!("123");
    sleep(Duration::from_secs(15)).await;

    Ok(())
}
