use crate::errors::{Error, ModelIndexError};
use futures::stream::StreamExt;
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// The URL of the public models index. This is maintained on a separate locked branch to prevent
/// changes to model locations from breaking every Sotto installation.
///
/// Note to contributors: only @arctic-hen7 has the authority to unlock the `prod-index` branch
/// to modify this index! Requests for changes should be made in issues, *not* PRs.
const INDEX_URL: &str =
    "https://raw.githubusercontent.com/arctic-hen7/sotto/prod-index/models.json";

/// The different kinds of models that can be downloaded in Sotto.
#[derive(Clone, Copy)]
pub enum Model {
    WhisperTiny,
    WhisperBase,
    WhisperSmall,
    WhisperMedium,
    WhisperLarge,
    // TODO TTS models
}
impl Model {
    /// A convenience method for core models that either gets them or downloads them without a prompt.
    /// This is intended for use with models whose presence is checked at startup.
    pub async fn get_or_download(&self) -> Result<PathBuf, Error> {
        if let Some(path) = self.get()? {
            Ok(path)
        } else {
            self.download().await
        }
    }
    /// Gets the path to this model, or returns `Ok(None)` if it hasn't been downloaded yet.
    pub fn get(&self) -> Result<Option<PathBuf>, Error> {
        // Make sure `~/.sotto` exists
        let home_dir = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        let sotto_dir = home_dir.join(".sotto");
        std::fs::create_dir_all(&sotto_dir)
            .map_err(|err| Error::CreateSottoDirFailed { source: err })?;

        let model_key = self.to_identifier();
        let download_path = sotto_dir.join(&format!("{model_key}.bin"));

        if download_path.exists() {
            Ok(Some(download_path))
        } else {
            Ok(None)
        }
    }
    /// Downloads this model. This will *not* check for the model's existence first, and should
    /// only be called if you're sure the desired model doesn't exist!
    pub async fn download(&self) -> Result<PathBuf, Error> {
        let client = Client::new();
        // Get the model index first and resolve the URL for the model
        let res = client
            .get(INDEX_URL)
            .send()
            .await
            .map_err(|err| Error::GetModelIndexFailed { source: err })?;
        if !res.status().is_success() {
            return Err(Error::GetModelIndexBadStatus {
                status: res.status().into(),
            });
        }
        let model_idx = res
            .text()
            .await
            .map_err(|err| Error::GetModelIndexFailed { source: err })?;
        // Represented in general terms to make future format changes eassy without needing to update
        // every Sotto installation on the planet
        let model_idx: HashMap<String, String> = serde_json::from_str(&model_idx)
            .map_err(|err| ModelIndexError::ParseFailed { source: err })?;
        // This is an error with the index, because it should support what's in a production app!
        let model_key = self.to_identifier();
        let model_url = model_idx
            .get(model_key)
            .ok_or(ModelIndexError::Incomplete {
                missing_key: model_key,
            })?;

        // Find the user's home directory in a cross-platform manner
        let home_dir = dirs::home_dir().ok_or(Error::NoHomeDir)?;
        let download_path = home_dir.join(".sotto").join(&format!("{model_key}.bin"));

        // Download the file
        let res = client
            .get(model_url)
            .send()
            .await
            .map_err(|err| Error::DownloadModelFailed { source: err })?;
        if !res.status().is_success() {
            return Err(Error::DownloadModelBadStatus {
                status: res.status().into(),
            });
        }

        // Stream the response into the target file (it's a model, it will be big)
        let mut file = File::create(&download_path)
            .await
            .map_err(|err| Error::CreateModelFileFailed { source: err })?;
        let mut body = res.bytes_stream();
        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|err| Error::BadChunk { source: err })?;
            file.write_all(&chunk)
                .await
                .map_err(|err| Error::WriteChunkFailed { source: err })?;
        }

        Ok(download_path)
    }
    fn to_identifier(&self) -> &'static str {
        match self {
            Self::WhisperTiny => "whisper_tiny",
            Self::WhisperBase => "whisper_base",
            Self::WhisperSmall => "whisper_small",
            Self::WhisperMedium => "whisper_medium",
            Self::WhisperLarge => "whisper_large",
        }
    }
}
