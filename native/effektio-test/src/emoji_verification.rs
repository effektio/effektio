use anyhow::Result;
use effektio::api::{login_new_client, EmojiVerificationEvent};
use futures::{channel::mpsc::Receiver, stream::StreamExt};
use log::info;
use tempfile::TempDir;

fn wait_for_emoji_verification_event(
    rx: &mut Receiver<EmojiVerificationEvent>,
    name: &str,
) -> EmojiVerificationEvent {
    loop {
        if let Ok(Some(event)) = rx.try_next() {
            if event.get_event_name().as_str() == name {
                return event;
            }
        }
    }
}

#[tokio::test]
async fn interactive_verification_started_from_request() -> Result<()> {
    let _ = env_logger::try_init();

    let alice_dir = TempDir::new()?;
    let alice = login_new_client(
        alice_dir.path().to_str().expect("always works").to_owned(),
        "@sisko:ds9.effektio.org".to_owned(),
        "sisko".to_owned(),
    )
    .await?;

    let alice_device_id = alice.device_id().await.expect("alice should get device id");
    info!("alice device id: {}", alice_device_id);

    let bob_dir = TempDir::new()?;
    let bob = login_new_client(
        bob_dir.path().to_str().expect("always works").to_owned(),
        "@sisko:ds9.effektio.org".to_owned(),
        "sisko".to_owned(),
    )
    .await?;

    let bob_device_id = bob.device_id().await.expect("bob should get device id");
    info!("bob device id: {}", bob_device_id);
    // we have two devices logged in

    // sync both up to ensure they've seen the other device
    let syncer = alice.start_sync();
    let mut first_synced = syncer.get_first_synced_rx().expect("not yet read");
    while first_synced.next().await != Some(true) {} // let's wait for it to have synced
    let mut alice_devices_changed_rx = syncer.get_devices_changed_event_rx().unwrap();
    let mut alice_rx = syncer.get_emoji_verification_event_rx().unwrap();

    let syncer = bob.start_sync();
    let mut first_synced = syncer.get_first_synced_rx().expect("not yet read");
    while first_synced.next().await != Some(true) {} // let's wait for it to have synced
    let mut bob_rx = syncer.get_emoji_verification_event_rx().unwrap();

    // according to alice bob is not verfied:
    assert!(!alice.verified_device(bob_device_id.clone()).await?);

    // according to bob alice is not verfied:
    assert!(!bob.verified_device(alice_device_id).await?);

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice gets notified that new device (Bob) was logged in
    loop {
        if let Ok(Some(event)) = alice_devices_changed_rx.try_next() {
            if let Ok(devices) = event.get_unverified_devices().await {
                for device in devices {
                    if !device.was_verified() {
                        info!("found device id: {}", device.get_device_id());
                    }
                }
                // Alice sends a verification request with her desired methods to Bob
                event
                    .request_verification_with_methods(
                        bob_device_id,
                        &mut vec!["m.sas.v1".to_owned()],
                    )
                    .await?;
                break;
            }
        }
    }

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob receives the request event from Alice
    let event = wait_for_emoji_verification_event(&mut bob_rx, "m.key.verification.request");

    // Bob accepts the request, sending a Ready request
    event
        .accept_verification_request_with_methods(&mut vec!["m.sas.v1".to_owned()])
        .await?;
    // And also immediately sends a start request
    let started = event.start_sas_verification().await?;
    assert!(started, "bob failed to start sas");

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice receives the ready event from Bob
    let event = wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.ready");

    // Alice immediately sends a start request
    let started = event.start_sas_verification().await?;
    assert!(started, "alice failed to start sas verification");

    // Now Alice receives the start event from Bob
    // Without this loop, sometimes the cancel event follows the start event
    wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.start");

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob receives the start event from Alice
    let event = wait_for_emoji_verification_event(&mut bob_rx, "m.key.verification.start");

    // Bob accepts it
    let accepted = event.accept_sas_verification().await?;
    assert!(accepted, "bob failed to accept sas verification");

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice receives the accept event from Bob
    let event = wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.accept");

    // Alice sends a key
    event.send_verification_key().await?;

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob receives the key event from Alice
    let bob_event = wait_for_emoji_verification_event(&mut bob_rx, "m.key.verification.key");

    // Bob gets the verification key from event
    let emoji_from_alice = bob_event.get_verification_emoji().await?;
    info!("emoji from alice: {:?}", emoji_from_alice);

    // Bob sends a key
    bob_event.send_verification_key().await?;

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice receives the key event from Bob
    let alice_event = wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.key");

    // Alice gets the verification key from event
    let emoji_from_bob = alice_event.get_verification_emoji().await?;
    info!("emoji from bob: {:?}", emoji_from_bob);

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob first confirms that the emojis match and sends the mac event...
    bob_event.confirm_sas_verification().await?;

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice first confirms that the emojis match and sends the mac event...
    alice_event.confirm_sas_verification().await?;

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob receives the mac event from Alice
    wait_for_emoji_verification_event(&mut bob_rx, "m.key.verification.mac");

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice receives the mac event from Bob
    wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.mac");

    // ----------------------------------------------------------------------------
    // On Bob's device:

    // Bob receives the done event from Alice
    wait_for_emoji_verification_event(&mut bob_rx, "m.key.verification.done");

    // ----------------------------------------------------------------------------
    // On Alice's device:

    // Alice receives the done event from Bob
    wait_for_emoji_verification_event(&mut alice_rx, "m.key.verification.done");

    Ok(())
}
