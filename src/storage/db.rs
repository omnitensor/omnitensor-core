// OmniTensor-Project/omnitensor-core/src/storage/db.rs

use rocksdb::{DB, Options, IteratorMode, WriteBatch};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Key not found: {0}")]
    KeyNotFound(Vec<u8>),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub struct Database {
    db: Arc<Mutex<DB>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn get<K, V>(&self, key: &K) -> Result<Option<V>>
    where
        K: Serialize,
        V: for<'de> Deserialize<'de>,
    {
        let key_bytes = bincode::serialize(key)?;
        let db = self.db.lock().await;
        match db.get(&key_bytes)? {
            Some(value_bytes) => Ok(Some(bincode::deserialize(&value_bytes)?)),
            None => Ok(None),
        }
    }

    pub async fn put<K, V>(&self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        let key_bytes = bincode::serialize(key)?;
        let value_bytes = bincode::serialize(value)?;
        let db = self.db.lock().await;
        db.put(&key_bytes, &value_bytes)?;
        Ok(())
    }

    pub async fn delete<K>(&self, key: &K) -> Result<()>
    where
        K: Serialize,
    {
        let key_bytes = bincode::serialize(key)?;
        let db = self.db.lock().await;
        db.delete(&key_bytes)?;
        Ok(())
    }

    pub async fn batch_write<K, V>(&self, data: &[(K, V)]) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        let mut batch = WriteBatch::default();
        for (key, value) in data {
            let key_bytes = bincode::serialize(key)?;
            let value_bytes = bincode::serialize(value)?;
            batch.put(&key_bytes, &value_bytes);
        }
        let db = self.db.lock().await;
        db.write(batch)?;
        Ok(())
    }

    pub async fn prefix_scan<K, V>(&self, prefix: &K) -> Result<Vec<(K, V)>>
    where
        K: Serialize + for<'de> Deserialize<'de>,
        V: for<'de> Deserialize<'de>,
    {
        let prefix_bytes = bincode::serialize(prefix)?;
        let db = self.db.lock().await;
        let iter = db.iterator(IteratorMode::From(&prefix_bytes, rocksdb::Direction::Forward));
        let mut results = Vec::new();

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(&prefix_bytes) {
                break;
            }
            let deserialized_key: K = bincode::deserialize(&key)?;
            let deserialized_value: V = bincode::deserialize(&value)?;
            results.push((deserialized_key, deserialized_value));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db = Database::new(temp_dir.path())?;

        // Test put and get
        db.put(&"key1", &"value1").await?;
        let value: Option<String> = db.get(&"key1").await?;
        assert_eq!(value, Some("value1".to_string()));

        // Test delete
        db.delete(&"key1").await?;
        let value: Option<String> = db.get(&"key1").await?;
        assert_eq!(value, None);

        // Test batch write
        let data = vec![
            ("key2", "value2"),
            ("key3", "value3"),
            ("key4", "value4"),
        ];
        db.batch_write(&data).await?;

        // Test prefix scan
        let results: Vec<(String, String)> = db.prefix_scan(&"key").await?;
        assert_eq!(results.len(), 3);
        assert!(results.contains(&("key2".to_string(), "value2".to_string())));
        assert!(results.contains(&("key3".to_string(), "value3".to_string())));
        assert!(results.contains(&("key4".to_string(), "value4".to_string())));

        Ok(())
    }
}