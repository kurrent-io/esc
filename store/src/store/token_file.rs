use crate::errors::{Result, StoreError};
use esc_client_base::Token;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug)]
pub struct TokenFile {
    file_path: PathBuf,
}

impl TokenFile {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub async fn load(&self) -> Result<Option<Token>> {
        if fs::metadata(&self.file_path).await.is_ok() {
            let token_bytes = fs::read(&self.file_path).await.map_err(|err| {
                StoreError::new("error loading token file")
                    .details(format!("file = {:?}", self.file_path))
                    .source(Box::new(err))
            })?;
            let token: Token = serde_json::from_slice(&token_bytes).map_err(|err| {
                StoreError::new("can't load token file as it is malformed")
                    .details(format!("file = {:?}", self.file_path))
                    .source(Box::new(err))
            })?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    pub async fn save(&mut self, token: Token) -> Result<Token> {
        let new_token_bytes = serde_json::to_vec(&token).map_err(|err| {
            StoreError::new("error saving token: serialization failure").source(Box::new(err))
        })?;

        let token_dir = match self.file_path.parent() {
            Some(dir) => dir,
            None => {
                return Err(StoreError::new(
                    "error saving token: the given path is not located in a directory",
                )
                .details(format!("{:?}", self.file_path)))
            }
        };
        tokio::fs::create_dir_all(&token_dir).await.map_err(|err| {
            StoreError::new_detailed(
                "error saving token: the destination directory could not be created",
                format!("directory={:?}", token_dir),
                Box::new(err),
            )
        })?;

        fs::write(&self.file_path, &new_token_bytes)
            .await
            .map_err(|err| {
                StoreError::new("error saving token: couldn't write file")
                    .details(format!("file = {:?}", self.file_path))
                    .source(Box::new(err))
            })?;
        Ok(token)
    }

    // delete removes the token file. Returns true if a file existed and was removed.
    pub async fn delete(&self) -> Result<bool> {
        match fs::remove_file(&self.file_path).await {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(StoreError::new("error removing token file")
                .details(format!("file = {:?}", self.file_path))
                .source(Box::new(err))),
        }
    }
}
