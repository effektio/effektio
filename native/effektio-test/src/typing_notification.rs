use anyhow::Result;
use effektio::api::login_new_client;
use futures::stream::StreamExt;
use tempfile::TempDir;

#[tokio::test]
async fn kyra_detects_sisko_typing() -> Result<()> {
    let _ = env_logger::try_init();

    let tmp_dir = TempDir::new()?;
    let sisko = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@sisko:ds9.effektio.org".to_owned(),
        "sisko".to_owned(),
    )
    .await?;
    let sisko_syncer = sisko.start_sync();
    let mut first_synced = sisko_syncer.get_first_synced_rx().expect("note yet read");
    while first_synced.next().await != Some(true) {} // let's wait for it to have synced
    let group = sisko
        .get_group("#ops:ds9.effektio.org".to_owned())
        .await
        .expect("sisko should belong to ops");
    let sent = group.typing_notice(true).await?;
    println!("sent: {:?}", sent);

    let tmp_dir = TempDir::new()?;
    let kyra = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@kyra:ds9.effektio.org".to_owned(),
        "kyra".to_owned(),
    )
    .await?;
    let kyra_syncer = kyra.start_sync();
    let mut rx = kyra_syncer.get_typing_notification_rx().unwrap();

    loop {
        match rx.try_next() {
            Ok(Some(event)) => {
                println!("received: {:?}", event);
                break;
            }
            Ok(None) => {
                println!("received: none");
            }
            Err(e) => {}
        }
    }

    Ok(())
}
