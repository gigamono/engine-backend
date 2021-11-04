use log::info;
use std::path::PathBuf;
use tokio::fs;
use utilities::{
    config::GigamonoConfig,
    errors,
    natsio::Payload,
    result::{Context, Result},
};

pub struct WorkspacePaths {
    canon_w_path: PathBuf,
    url_path: String,
}

impl WorkspacePaths {
    pub async fn new(payload: &Payload, config: &GigamonoConfig) -> Result<Self> {
        // Get the root path
        let root_path = &config.engines.backend.root_path;

        // Construct canonical workspace path.
        let full_path = format!("{}/workspaces/{}", root_path, payload.workspace_id);
        let canon_w_path = fs::canonicalize(&full_path).await.context(format!(
            r#"getting canonical workspace path from, "{}""#,
            full_path
        ))?;

        info!("Canonical workspace path {:?}", canon_w_path);

        // Get stripped url path.
        let mut url_path = payload.request.path.as_str();
        if let Some(stripped_path) = url_path.strip_prefix("/r/") {
            url_path = stripped_path;
        }

        info!("Path suffix {:?}", url_path);

        Ok(Self {
            canon_w_path: canon_w_path,
            url_path: url_path.to_owned(),
        })
    }

    pub async fn get_canon_path(&self, relative_path: &str) -> Result<PathBuf> {
        // Construct canonical path.
        let full_path = format!("{}/{}", self.canon_w_path.display(), relative_path);
        let resolved_path = fs::canonicalize(&full_path).await.context(format!(
            r#"getting canonical path of a workspace-relative path from, "{}""#,
            full_path
        ))?;

        // SEC: Making sure canon_w_path is still base.
        if !resolved_path.starts_with(&self.canon_w_path) {
            errors::any_error(format!(
                r#"path {:?} must be a under of workspace path {:?}"#,
                resolved_path, self.canon_w_path,
            ))?;
        }

        Ok(resolved_path)
    }

    pub async fn get_canon_path_from_surl(&self, relative_path: &str) -> Result<PathBuf> {
        self.get_canon_path(&format!("/surl/{}/{}", self.url_path, relative_path))
            .await
    }

    pub fn get_relative_path_from_surl(&self, relative_path: &str) -> String {
        format!("/surl/{}/{}", self.url_path, relative_path)
    }
}
