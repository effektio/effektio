use super::room::Room;
use effektio_core::executor::Executor;
use futures::{pin_mut, StreamExt};
use ruma::OwnedEventId;

pub struct Group {
    pub(crate) inner: Room,
}

impl Group {
    pub async fn sync_up(&self) -> anyhow::Result<()> {
        let store = &*self.inner.client.store();
        let sync_point_key_str = format!("sync-point::{}", self.inner.room_id());
        let sync_point_key = sync_point_key_str.as_bytes();
        let sync_point: Option<OwnedEventId> = store
            .get_custom_value(sync_point_key)
            .await?
            .map(|e| bincode::deserialize(&e))
            .transpose()?;
        let (fwd_stream, backward_stream) = (*self.inner).timeline().await?;
        let mut to_execute = vec![];
        pin_mut!(backward_stream);
        loop {
            match backward_stream.next().await {
                Some(Ok(event)) => {
                    if sync_point.is_some() && event.event_id() == sync_point {
                        // we are done catching up, just break the loop
                        break;
                    }
                }
                Some(Ok(event)) => {
                    // unknown event found, add to list
                    to_execute.push(event)
                }
                Some(Err(e)) => {
                    // FIXME make this more specific
                    // we are done with the loop
                    log::error!("Failed to read back in time: {:}", e);
                    break;
                }
                None => {
                    // end
                    break;
                }
            }
        }

        // we need to turn it around.
        to_execute.reverse();
        let executor = Executor::new(&**self.inner.client.store(), self.inner.room_id().to_owned());

        for e in to_execute {
            executor.apply(&e).await?;
            store.set_custom_value(sync_point_key, bincode::serialize(&e.event_id())?);
        }

        pin_mut!(fwd_stream);
        while let Some(event) = fwd_stream.next().await {
            // unknown event found, add to list
            executor.apply(&event).await?;
            store.set_custom_value(sync_point_key, bincode::serialize(&event.event_id())?);
        }

        Ok(())
    }
}

impl std::ops::Deref for Group {
    type Target = Room;
    fn deref(&self) -> &Room {
        &self.inner
    }
}
