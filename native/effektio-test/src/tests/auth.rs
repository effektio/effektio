use anyhow::Result;
use effektio::api::{guest_client, login_new_client, login_with_token};
use tempfile::TempDir;

#[tokio::test]
async fn can_guest_login() -> Result<()> {
    let _ = env_logger::try_init();
    let tmp_dir = TempDir::new()?;
    let _client = guest_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        option_env!("HOMESERVER")
            .unwrap_or("http://localhost:8118")
            .to_string(),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn sisko_can_login() -> Result<()> {
    let _ = env_logger::try_init();
    let tmp_dir = TempDir::new()?;
    let _client = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@sisko:ds9.effektio.org".to_owned(),
        "sisko".to_owned(),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn kyra_can_login() -> Result<()> {
    let _ = env_logger::try_init();
    let tmp_dir = TempDir::new()?;
    let _client = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@kyra:ds9.effektio.org".to_owned(),
        "kyra".to_owned(),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn kyra_can_restore() -> Result<()> {
    let _ = env_logger::try_init();
    let tmp_dir = TempDir::new()?;
    let client = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@kyra:ds9.effektio.org".to_owned(),
        "kyra".to_owned(),
    )
    .await?;
    let token = client.restore_token().await?;
    let user_id = client.user_id().await?;
    drop(client);

    let client = login_with_token(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        token,
    )
    .await?;
    assert_eq!(client.user_id().await?, user_id);
    Ok(())
}
