// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::paths::WorkspacePaths;
use tokio::fs;
use utilities::{
    config::GigamonoConfig,
    result::{Context, Result},
};

pub struct FileManager {
    pub(crate) paths: WorkspacePaths,
}

impl FileManager {
    pub(crate) async fn new(
        workspace_id: &str,
        url_path: &str,
        config: &GigamonoConfig,
    ) -> Result<Self> {
        // Construct new workspace path.
        let paths = WorkspacePaths::new(workspace_id, url_path, config).await?;
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
