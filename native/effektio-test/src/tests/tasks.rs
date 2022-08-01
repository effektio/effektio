use anyhow::Result;
use effektio::api::login_new_client;
use tempfile::TempDir;

#[tokio::test]
async fn tasks_smoketest() -> Result<()> {
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

    let mut task_list_draft = ops
        .task_list_draft()
        .expect("we are in and admin, we can create news drafts");
    task_list_draft.name("Daily Standup".to_owned());
    let task_list_id = task_list_draft.send().await?;
    client
        .sync_once(Default::default())
        .await
        .expect("sync works");
    // we should have
    let task_list = client
        .task_lists()
        .await
        .into_iter()
        .next()
        .expect("we should have a task list");
    assert_eq!(
        task_list.event_id(),
        task_list_id,
        "Latest task list isn't the item, we just sent",
    );

    assert_eq!(
        task_list.tasks().len(),
        0,
        "There are already tasks in the new list"
    );

    let mut task_draft = task_list.task_builder();
    task_draft.title("Check in with station security".to_owned());
    let task_id = task_draft.send().await?;

    client
        .sync_once(Default::default())
        .await
        .expect("sync works");

    assert_eq!(task_list.tasks().len(), 1, "Task is on our list");
    Ok(())
}
