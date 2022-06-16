
use matrix_sdk_base::store::StateStore;
use std::sync::Arc;
use futures_signals::{
    signal_map::MutableBTreeMap,
    signal_vec::MutableVec,
};
use serde::{Serialize, de::DeserializeOwned};
use anyhow::{bail, Context, Result};

#[derive(Debug)]
pub struct Store {
    inner: Box<dyn StateStore>,
    indizes: MutableBTreeMap<String, MutableVec<String>>,
}

impl Store {
    pub fn new(inner: Box<dyn StateStore>) -> Store {
        Store {
            inner,
            indizes: Default::default(),
        }
    }

    pub async fn set_model<T>(&self, key: String, model: T, indizes: Vec<String>) -> Result<()>
    where
        T: Serialize,
    {
        let set_key = key.as_bytes();
        let value = rmp_serde::encode::to_vec(&model)?;
        self.inner.set_custom_value(set_key, value).await?;

        let mut updated = Vec::new();

        for idx_key in indizes {
            let mut idx_map = self.indizes.lock_mut();
            if let Some(vec_map) = idx_map.get(&idx_key) {
                let mut v = vec_map.lock_mut();
                if !v.contains(&key) {
                    v.push_cloned(key.clone());
                    updated.push(idx_key);
                }
            } else {
                // idx not yet loaded, refresh from inner stores
                let idx_store = format!("efk-idx-{}", idx_key);
                let inner_vec = if let Some(inner) = self.inner.get_custom_value(idx_store.as_bytes()).await? {
                    let mut v = rmp_serde::decode::from_slice::<Vec<String>>(&inner)?;
                    v.push(key.clone());
                    v
                } else  {
                    vec![key.clone()]
                };

                idx_map.insert_cloned(idx_key.clone(), MutableVec::new_with_values(inner_vec));
                updated.push(idx_key);
            }
        }

        if !updated.is_empty() {
            self.flush_indizes(updated).await?;
        }

        Ok(())
    }

    pub async fn get_model<T>(&self, key: String) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let set_key = key.as_bytes();
        if let Some(v) = self.inner.get_custom_value(key.as_bytes()).await? {
            Ok(Some(rmp_serde::decode::from_slice(&v)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_index(&self, key: String) -> Result<Option<Vec<String>>> {
        if let Some(vec_map) = self.indizes.lock_ref().get(&key) {
            return Ok(Some(vec_map.lock_ref().to_vec()))
        }

        // idx not yet loaded, refresh from inner stores
        let mut idx_map = self.indizes.lock_mut();
        let idx_store = format!("efk-idx-{}", key);
        if let Some(inner) = self.inner.get_custom_value(idx_store.as_bytes()).await? {
            let inner_vec = rmp_serde::decode::from_slice::<Vec<String>>(&inner)?;
            idx_map.insert_cloned(key, MutableVec::new_with_values(inner_vec.clone()));
            return Ok(Some(inner_vec))
        } else  {
            Ok(None)
        }
    }

    async fn flush_indizes(&self, names: Vec<String>)  -> Result<()> {
        let mapper = self.indizes.lock_ref();
        for name in names {
            let idx_key = format!("efk-idx-{}", name);
            self.inner.set_custom_value(
                idx_key.as_bytes(),
                rmp_serde::encode::to_vec(mapper.get(&name).context("Index unknown")?.lock_ref().as_slice())?
            ).await?;
        }

        Ok(())

    }
}


#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use matrix_sdk_base::store::MemoryStore;

    pub fn test_store() -> Store {
        Store::new(Box::new(MemoryStore::default()))

    }
}