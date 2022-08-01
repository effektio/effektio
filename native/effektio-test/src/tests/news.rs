use anyhow::Result;
use effektio::api::login_new_client;
use tempfile::TempDir;

#[tokio::test]
async fn sisko_posts_news() -> Result<()> {
    let _ = env_logger::try_init();
    let tmp_dir = TempDir::new()?;
    let client = login_new_client(
        tmp_dir.path().to_str().expect("always works").to_owned(),
        "@sisko:ds9.effektio.org".to_owned(),
        "sisko".to_owned(),
    )
    .await?;
    client
        .sync_once(Default::default())
        .await
        .expect("sync works");
    let ops = client
        .get_group("#ops:ds9.effektio.org".to_owned())
        .await
        .expect("Promenade exists");

    let news_draft = ops
        .news_draft()
        .expect("we are in, we can create news drafts");
    news_draft
        .add_text("This is a simple text example".to_owned())
        .await?;
    let event_id = news_draft.send().await?;
    client
        .sync_once(Default::default())
        .await
        .expect("sync works");
    // we should have
    let latest_news_item = client
        .latest_news()
        .await?
        .into_iter()
        .next()
        .expect("we should have a news item");
    // assert_eq!(
    //     latest_news_item.event_id(),
    //     event_id,
    //     "Latest news isn't the item, we just sent",
    // );

    Ok(())
}
