use crate::WorkspacePaths;
use tokio::fs;
use utilities::{config::GigamonoConfig, natsio::Payload, result::{Result, Context}};

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
            .context(format!(r#"attempt to read file, "{:?}""#, path))
    }

    pub async fn read_file_from_surl(&self, relative_path: &str) -> Result<String> {
        let path = self.paths.get_canon_path_from_surl(relative_path).await?;

        fs::read_to_string(&path)
            .await
            .context(format!(r#"attempt to read file, "{:?}""#, path))
    }
}
