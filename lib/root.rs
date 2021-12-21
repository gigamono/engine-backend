// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::{fs, path::PathBuf};
use utilities::{
    errors,
    result::{Context, Result},
};

#[derive(Clone)]
pub struct RootManager {
    canon_workspace_path: PathBuf,
}

pub enum RootLevel {
    Workspace,
    Api,
    Apps,
    Extensions,
    Scheduled,
}

impl RootManager {
    pub fn new(backend_root: &str, workspace_id: &str) -> Result<Self> {
        let workspace_path = &format!("{}/workspaces/{}", backend_root, workspace_id);
        let canon_workspace_path = fs::canonicalize(workspace_path).context(format!(
            r#"getting canonical workspace path from, "{}""#,
            workspace_path
        ))?;

        Ok(Self {
            canon_workspace_path,
        })
    }

    pub fn read_file_from(&self, path: &str, level: RootLevel) -> Result<String> {
        self.read_file(&format!(
            "{}/{}/{}",
            self.canon_workspace_path.display(),
            level.get_path(),
            path
        ))
    }

    pub fn read_file_from_workspace(&self, path: &str) -> Result<String> {
        self.read_file(&format!("{}/{}", self.canon_workspace_path.display(), path))
    }

    fn read_file(&self, full_path: &str) -> Result<String> {
        // Validate path.
        let canon_path = self.validate_path(full_path)?;

        // Read file.
        fs::read_to_string(&canon_path)
            .context(format!(r#"attempt to read file, "{:?}""#, canon_path))
    }

    fn validate_path(&self, path: &str) -> Result<PathBuf> {
        // Canonicalize path.
        let canon_path = fs::canonicalize(path)
            .context(format!(r#"getting canonical path from, "{}""#, path))?;

        // SEC: Making sure workspace paths is still base.
        if !canon_path.starts_with(&self.canon_workspace_path) {
            return errors::new_error_t(format!(
                r#"path {:?} must be a under of workspace path {:?}"#,
                path, self.canon_workspace_path,
            ));
        }

        Ok(canon_path)
    }
}

impl RootLevel {
    pub fn get_path(&self) -> String {
        match self {
            RootLevel::Workspace => String::new(),
            RootLevel::Api => String::from("api"),
            RootLevel::Apps => String::from("apps"),
            RootLevel::Extensions => String::from("extensions"),
            RootLevel::Scheduled => String::from("scheduled"),
        }
    }
}
