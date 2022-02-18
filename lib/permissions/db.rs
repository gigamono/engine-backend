// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use log::debug;
use regex::Regex;
use std::any::TypeId;
use tera::permissions::{PermissionType, PermissionTypeKey, Resource, State};
use utilities::{errors, result::Result};

/// The database permissions.
#[derive(Debug, Clone)]
pub enum Db {
    Connect,
    DatabaseCreate,
    DatabaseDelete,
    TableCreate,
    TableDelete,
    ColumnCreate,
    ColumnDelete,
    RowCreate,
    RowDelete,
    RowRead,
    RowWrite,
}

/// A [`database path`](struct@DbPath) can be namespaced. DbRoot can be used to represent such namespace.
///
/// If no namespace is expected, DbRoot should have an empty string as value.
///
/// In Gigamono's case, a namespace is an associated workspace identifier.
#[derive(Clone, Debug)]
pub struct DbRoot(String);

/// DbPath uses an addressing scheme similar to a URL that tells us what resource needs to be accessed.
///
/// This is what the scheme looks: `/database[/table[/column]]`.
/// The table and column components are optional depending on context.
///
/// The database name is a UTF-8 string that cannot be longer than 48 characters.
/// MySQL supports up to 64 characters as db name. The remaining 16 characters are reserved for other uses.
///
/// In Gigamono's case, the first 15 characters represent the workspace identifier. The 16th character is going to be the `$` separator character.
#[derive(Clone, Debug)]
pub struct DbPath {
    string: String,
    regex: Option<Regex>,
}

impl Db {
    /// Appends the root to the path delimited by `$` character.
    pub fn append_root(root: &String, path: &String) -> String {
        if !root.is_empty() {
            format!("{}${}", root, path)
        } else {
            path.clone()
        }
    }
}

impl PermissionType for Db {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }

    fn map(
        &self,
        allow_list: Vec<Box<dyn Resource>>,
        state: &Option<Box<dyn tera::permissions::State>>,
    ) -> Result<Vec<Box<dyn Resource>>> {
        let list = allow_list
            .iter()
            .map(|dir| {
                // Downcast state to Root. Expects a root to be specified.
                let root = if let Some(state) = &state {
                    state.downcast_ref::<DbRoot>().unwrap().as_ref()
                } else {
                    return errors::permission_error_t("root namespace not specified");
                };

                // Get path.
                let path_string = dir.downcast_ref::<DbPath>().unwrap().as_ref();

                // Ensure db name part of the scheme is not larger than 48 characters.
                let db_name = {
                    let trimmed = path_string.trim_start_matches("/");
                    trimmed.split_once("/").unwrap_or((trimmed, "")).0
                };

                if db_name.chars().count() > 48 {
                    return errors::new_error_t(format!("database name is too long: {}", db_name));
                }

                // Append root.
                let full_path_string = Self::append_root(root, path_string);

                debug!("Allowed path = {:?}", full_path_string);

                // SEC: Create regex that allows patterns like these:
                // https://gist.github.com/appcypher/7074d219493fa2711c36b2d19fe75eb9#file-patterns-md
                let pattern = full_path_string.replace("**", r".+").replace("*", r"[^/]+");

                // SEC: Ensuring the pattern matches against the whole string.
                let re = Regex::new(&format!(r"^{}$", pattern)).unwrap();

                let db_path = DbPath {
                    string: full_path_string.clone(),
                    regex: Some(re),
                };

                Ok(db_path.into())
            })
            .collect::<Result<Vec<Box<dyn Resource>>>>()?;

        Ok(list)
    }

    fn check(
        &self,
        path: &Box<dyn Resource>,
        allow_list: std::rc::Rc<Vec<Box<dyn Resource>>>,
        state: &Option<Box<dyn tera::permissions::State>>,
    ) -> Result<()> {
        // Downcast state to Root. Expects a root to be specified.
        let root = if let Some(state) = &state {
            state.downcast_ref::<DbRoot>().unwrap().as_ref()
        } else {
            return errors::permission_error_t("root namespace not specified");
        };

        // Get path.
        let path_string = path.downcast_ref::<DbPath>().unwrap().as_ref();

        // Append root.
        let full_path_string = Self::append_root(root, path_string);

        for allowed_path in allow_list.iter() {
            // Downcast trait object to Path.
            let db_path = allowed_path.downcast_ref::<DbPath>().unwrap();

            // SEC: Check if path matches pattern.
            if db_path.regex.as_ref().unwrap().is_match(&full_path_string) {
                return Ok(());
            }
        }

        errors::permission_error_t(format!(
            r#"permission type "{}" does not exist for scheme {:?}"#,
            self.get_type(),
            path
        ))
    }
}

impl Resource for DbPath {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbPath")
            .field("string", &self.string)
            .field("regex", &self.regex)
            .finish()
    }

    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl State for DbRoot {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DbRoot").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for Db {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for DbPath {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}

impl AsRef<String> for DbPath {
    fn as_ref(&self) -> &String {
        &self.string
    }
}

impl From<&str> for DbPath {
    fn from(path: &str) -> Self {
        Self {
            string: path.into(),
            regex: None,
        }
    }
}

impl From<&String> for DbPath {
    fn from(path: &String) -> Self {
        Self {
            string: path.into(),
            regex: None,
        }
    }
}

impl From<String> for DbPath {
    fn from(path: String) -> Self {
        Self {
            string: path.into(),
            regex: None,
        }
    }
}

impl From<&str> for DbRoot {
    fn from(root: &str) -> Self {
        Self(root.into())
    }
}

impl From<&String> for DbRoot {
    fn from(root: &String) -> Self {
        Self(root.into())
    }
}

impl From<String> for DbRoot {
    fn from(root: String) -> Self {
        Self(root)
    }
}

impl AsRef<String> for DbRoot {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
