// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{
    cell::RefCell,
    path::{self, Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::{root::RootManager, runtimes::ApiPermissions};
use log::debug;
use regex::Regex;
use tera::{
    events::{Events, HttpResponder},
    permissions::Permissions,
    Runtime, RuntimeOptions,
};
use tokio::sync::mpsc::Sender;
use utilities::{
    config::ApiManifest,
    errors, http,
    hyper::{Body, Method, Request, Response},
    result::Result,
    setup::CommonSetup,
};

/// A runtime for executing previously-defined scripts and modules relating to an api.
///
/// The runtime expects to find the request url path mapped directly to a similar-looking path in the workspace root.
pub struct ApiRuntime {
    relative_folder_path: String,
    root_mgr: RootManager,
    manifest: ApiManifest,
    runtime: Runtime,
    method: Method,
}

impl ApiRuntime {
    /// Creates a new API runtime.
    pub async fn new(
        request: Request<Body>,
        response_tx: Rc<Sender<Response<Body>>>,
        setup: Arc<CommonSetup>,
    ) -> Result<Self> {
        // Get config.
        let config = &setup.config;

        // Get workspace id.
        let workspace_id = http::get_header_value(&request, http::WORKSPACE_ID_HEADER)?;

        // Get url path.
        let url_path = request.uri().path().to_string();

        // Get request method. Used to determine the index script to run.
        let method = request.method().to_owned();

        debug!("Request path = {}", url_path);

        // Create root manager.
        let root_mgr = RootManager::new(&config.engines.backend.root_path, &workspace_id)?;

        // Resolve path params.
        let relative_folder_path = Self::resolve_url_path(&url_path)?;

        debug!("Resolved url path = {}", &relative_folder_path);

        // Create events.
        let events = Rc::new(RefCell::new(Events {
            http: Some(tera::events::HttpEvent::new(
                request,
                Rc::new(HttpResponder::new(response_tx)),
            )),
        }));

        // Parse the api manifest.
        let content = root_mgr.read_file_from_workspace(
            &[&relative_folder_path, "api.yaml"]
                .iter()
                .collect::<PathBuf>(),
        )?;

        // Parse manifest.
        let manifest = ApiManifest::try_from(&content)?;

        // Get permissions.
        let permissions =
            ApiPermissions::load_permissions(&manifest, &root_mgr.canon_workspace_path)?;

        // TODO(appcypher): ...
        // Get custom postcripts.
        let custom_postscripts = vec![];

        // Create runtime.
        let runtime = Runtime::with_events(
            permissions,
            events,
            config.engines.backend.runtime.enable_snapshot,
            custom_postscripts,
            RuntimeOptions {
                ..Default::default()
            },
        )
        .await?;

        Ok(Self {
            relative_folder_path,
            root_mgr,
            manifest,
            runtime,
            method,
        })
    }

    /// Executes the auth script (if enabled), the middleware scripts and the associated index module of the api.
    pub async fn execute(&mut self) -> Result<bool> {
        // Run auth if enabled.
        if self.manifest.authentication.enabled {
            if !self.run_auth().await? {
                return Ok(false);
            };
        }

        // Run middlewares
        if !self.run_middlewares().await? {
            return Ok(false);
        };

        // Run index.
        self.run_index().await?;

        Ok(true)
    }

    /// Executes the auth script.
    async fn run_auth(&mut self) -> Result<bool> {
        // TODO(appcypher): Permissions.
        let filepath = "auth.js";

        // Scripts are not modules so they all share scopes.
        // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
        let code = Self::format_code(
            &self
                .root_mgr
                .read_file_from_workspace(&PathBuf::from(filepath))?,
        );

        // TODO(appcypher): Get permissions from config.
        let permissions = Permissions::default();

        // Execute script.
        let value_global = self
            .runtime
            .execute_middleware_script(filepath, code, permissions)
            .await?;

        let scope = &mut self.runtime.handle_scope();
        let value = value_global.open(scope);

        Ok(value.boolean_value(scope))
    }

    /// Executes the middleware scripts defined in the api manifest.
    async fn run_middlewares(&mut self) -> Result<bool> {
        // TODO(appcypher):  Permissions.
        for path in self.manifest.middlewares.iter() {
            let filepath = &path.script;

            // Scripts are not modules so they all share scopes.
            // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
            let code = Self::format_code(
                &self
                    .root_mgr
                    .read_file_from_workspace(&PathBuf::from(filepath))?,
            );

            // TODO(appcypher): Get permissions from config.
            let permissions = Permissions::default();

            // Execute script.
            let value_global = self
                .runtime
                .execute_middleware_script(filepath, code, permissions)
                .await?;

            let scope = &mut self.runtime.handle_scope();
            let value = value_global.open(scope);

            if !value.boolean_value(scope) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Executes the index module that corresponds to the api in topic.
    pub async fn run_index(&mut self) -> Result<()> {
        // Get specialised path for http method or just "index.js" if it does not exist.
        let filepath = match self.method {
            Method::GET => self.get_method_index_path("get"),
            Method::POST => self.get_method_index_path("post"),
            Method::PUT => self.get_method_index_path("put"),
            Method::DELETE => self.get_method_index_path("delete"),
            Method::HEAD => self.get_method_index_path("head"),
            Method::OPTIONS => self.get_method_index_path("options"),
            Method::CONNECT => self.get_method_index_path("connect"),
            Method::PATCH => self.get_method_index_path("patch"),
            Method::TRACE => self.get_method_index_path("trace"),
            _ => [&self.relative_folder_path, "index.js"].iter().collect(),
        };

        debug!("Index relative filepath = {:?}", filepath);

        // Grab code from file.
        let code = &self.root_mgr.read_file_from_workspace(&filepath)?;

        // Make index path absolute.
        let abs_path: PathBuf = [&PathBuf::from(path::MAIN_SEPARATOR.to_string()), &filepath]
            .iter()
            .collect();

        debug!("Index absolute filepath = {:?}", abs_path);

        // Execute module.
        self.runtime
            .execute_module(abs_path.display().to_string(), code)
            .await?;

        Ok(())
    }

    /// Gets a specialised index module path based on the request's http method.
    /// For example, if the request has a GET method, the index module will be `"index.get.js"` or `"index.js"` if that does not exist.
    ///
    /// Falls back to `"index.js"` if specialised path does not exist.
    fn get_method_index_path(&self, method: &str) -> PathBuf {
        let relative_path: PathBuf = [&self.relative_folder_path, &format!("index.{}.js", method)]
            .iter()
            .collect();

        let full_path: PathBuf = [&self.root_mgr.canon_workspace_path, &relative_path]
            .iter()
            .collect();

        // Check that full path exists.
        if Path::new(&full_path).exists() {
            relative_path
        } else {
            [&self.relative_folder_path, "index.js"].iter().collect()
        }
    }

    /// Adds code string within an iife syntax to prevent accidental leak of data to global space.
    fn format_code(code: &str) -> String {
        // SEC: Note that there still ways to leak things into the global scope. https://gist.github.com/appcypher/2c210cd04774f1812a4b3e5c84496858
        // Not sure if this is a critical security issue yet.
        format!("\"use strict\"; (\n{} \n)();", code)
    }

    /// Converts url path to platform path and resolves path params in path.
    ///
    /// If a path ends with a param path `=`, the parent is returned instead.
    fn resolve_url_path(url_path: &str) -> Result<String> {
        let platform_path = &Self::to_platform_path(url_path)?;

        // SEC: Get regex pattern of current platform's separator.
        let re_sep = utilities::path::get_platform_sep_pattern();

        // Pattern that matches path param like `\=foo\` in `C:\\Users\=foo\name`.
        let pattern = format!(r"{}=[^{}]*{}?", re_sep, re_sep, re_sep);
        let re = Regex::new(&pattern).unwrap();

        // Replace all path param pattern with "\=\" (in unix for example)
        let replace = format!(r"{}={}", path::MAIN_SEPARATOR, path::MAIN_SEPARATOR);
        let resolved_param_path = re.replace_all(platform_path, replace);

        debug!("Resolved param path = {}", &resolved_param_path);

        // Remove trailing `=` in path. This is because a resolved path that ends with `=` should be handled by its parent directory.
        // NOTE: SEC: Since we are only trimming, it is not possible to `../{workspace_root}`.
        let resolved_param_path = resolved_param_path
            .trim_end_matches(path::MAIN_SEPARATOR)
            .trim_end_matches('=');

        // Finally remove any path separator at both ends of the path.
        Ok(resolved_param_path
            .trim_end_matches(path::MAIN_SEPARATOR)
            .trim_start_matches(path::MAIN_SEPARATOR)
            .to_string())
    }

    /// Converts url path to platform path (using the platform's main separator)
    ///
    /// Windows path separator is not allowed in url.
    fn to_platform_path(url_path: &str) -> Result<String> {
        // SEC: Check if there is windows path separator in the url.
        if url_path.contains(r"\") {
            return errors::new_error_t(r"the `\` character is not supported in a url");
        }

        if !cfg!(unix) {
            // Replace url separators with platform-specific separators.
            return Ok(url_path.replace("/", &path::MAIN_SEPARATOR.to_string()));
        }

        return Ok(url_path.to_string());
    }
}
