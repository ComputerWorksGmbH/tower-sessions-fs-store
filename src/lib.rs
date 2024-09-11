use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tower_sessions::{
    session::{Id, Record},
    session_store::{Error, Result},
    SessionStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStore {
    base_path: &'static str,
}

impl FileStore {
    pub fn new(base_path: &'static str) -> Self {
        Self { base_path }
    }

    async fn record_path(&self, id: &Id) -> Result<String> {
        fs::create_dir_all(self.base_path).await.map_err(|_| {
            Error::Backend(format!(
                "failed to create directory with the path '{}'",
                self.base_path
            ))
        })?;

        Ok(format!("{}/{id}.json", self.base_path))
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self {
            base_path: "sessions",
        }
    }
}

#[async_trait]
impl SessionStore for FileStore {
    async fn create(&self, record: &mut Record) -> Result<()> {
        let path = self.record_path(&record.id).await?;

        let json_data = record_to_json(record)?;

        fs::write(path, json_data)
            .await
            .map_err(|_| Error::Backend("failed to write record to file".to_string()))?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> Result<()> {
        let path = self.record_path(&record.id).await?;

        let json_data = record_to_json(record)?;

        fs::write(path, json_data)
            .await
            .map_err(|_| Error::Backend("failed to write record to file".to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let path = self.record_path(session_id).await?;

        let record = load_record(&path).await;

        Ok(record)
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        let path = self.record_path(session_id).await?;

        fs::remove_file(path)
            .await
            .map_err(|_| Error::Backend("cannot delete record file".to_string()))?;

        Ok(())
    }
}

// broken records are counted as the record does not exist
async fn load_record(path: &str) -> Option<Record> {
    let json = fs::read(path).await.ok()?;
    let record: Record = serde_json::from_slice(&json).ok()?;

    Some(record)
}

fn record_to_json(record: &Record) -> Result<String> {
    serde_json::to_string_pretty(record)
        .map_err(|_| Error::Encode("failed to serialize record to json".to_string()))
}
