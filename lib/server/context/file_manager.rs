use crate::WorkspacePaths;
use tokio::fs;
use utilities::{
    config::GigamonoConfig, messages::error::SystemError, natsio::Payload, result::Result,
};

pub struct FileManager {
    pub(crate) paths: WorkspacePaths,
}

impl FileManager {
    pub(crate) async fn new(payload: &Payload, config: &GigamonoConfig) -> Result<Self> {
        // Construct new workspace path.
        let paths = WorkspacePaths::new(payload, config).await?;
        Ok(Self { paths })
    }

    pub async fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.paths.get_canon_path(relative_path).await?;

        fs::read_to_string(&path)
            .await
            .map_err(|err| SystemError::Io {
                ctx: format!(r#"attempt to read file, "{:?}""#, path),
                src: err,
            })
    }

    pub async fn read_file_from_surl(&self, relative_path: &str) -> Result<String> {
        let path = self.paths.get_canon_path_from_surl(relative_path).await?;

        fs::read_to_string(&path)
            .await
            .map_err(|err| SystemError::Io {
                ctx: format!(r#"attempt to read file, "{:?}""#, path),
                src: err,
            })
    }
}
