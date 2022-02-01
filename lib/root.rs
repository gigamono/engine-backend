// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{
    fs,
    path::{Path, PathBuf},
};
use utilities::{
    errors,
    result::{Context, Result},
};

/// Manages files in the workspace root.
#[derive(Clone)]
pub struct RootManager {
    pub canon_workspace_path: PathBuf,
}

/// Common paths that are relative to the workspace root.
pub enum RootLevel {
    Api,
    ApiSystem,
    Apps,
    Extensions,
    Scheduled,
}

impl RootManager {
    /// Creates a new root manager.
    pub fn new(root: &str, workspace_id: &str) -> Result<Self> {
        let workspace_path: PathBuf = [root, "workspaces", workspace_id].iter().collect();

        let canon_workspace_path = fs::canonicalize(&workspace_path).context(format!(
            r#"getting canonical workspace path from {:?}"#,
            workspace_path
        ))?;

        Ok(Self {
            canon_workspace_path,
        })
    }

    /// Reads file from a path realative to `level`.
    ///
    /// Does not want specified path to be preceded by a path separator.
    pub fn read_file_from(&self, path: &Path, level: RootLevel) -> Result<String> {
        // Join paths.
        let path: PathBuf = [
            &self.canon_workspace_path,
            &level.get_path(),
            &PathBuf::from(path),
        ]
        .iter()
        .collect();

        self.read_file(&path)
    }

    /// Reads file from a path relative to the workspace root.
    ///
    /// Does not want specified path to be preceded by a path separator.
    pub fn read_file_from_workspace(&self, path: &Path) -> Result<String> {
        // Join paths.
        let path: PathBuf = [&self.canon_workspace_path, &PathBuf::from(path)]
            .iter()
            .collect();

        self.read_file(&path)
    }

    /// Reads file from a path.
    ///
    /// Expects an absolute path.
    fn read_file(&self, full_path: &Path) -> Result<String> {
        // Validate path.
        let canon_path = self.validate_path(full_path)?;

        // Read file.
        fs::read_to_string(&canon_path).context(format!(r#"attempt to read file {:?}"#, canon_path))
    }

    /// Checks that specified path is still within the workspace root.
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // SEC: Canonicalize path.
        let canon_path =
            fs::canonicalize(path).context(format!(r#"getting canonical path from {:?}"#, path))?;

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
    /// Gets the corresponding path (relative to workspace root) of specified variant.
    pub fn get_path(&self) -> PathBuf {
        match self {
            RootLevel::Api => PathBuf::from("api"),
            RootLevel::ApiSystem => ["api", "system"].iter().collect::<PathBuf>(),
            RootLevel::Apps => PathBuf::from("apps"),
            RootLevel::Extensions => PathBuf::from("extensions"),
            RootLevel::Scheduled => PathBuf::from("scheduled"),
        }
    }
}
