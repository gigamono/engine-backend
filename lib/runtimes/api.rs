// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::{cell::RefCell, rc::Rc};

use crate::root::RootManager;
use tera::{
    events::Events,
    permissions::{
        events::event_http::{HttpEvent, HttpEventPath},
        fs::{Fs, FsPath, FsRoot},
        Permissions,
    },
    Runtime,
};
use utilities::{config::ApiManifest, result::Result};

pub struct ApiRuntime {
    root_mgr: RootManager,
    api_path: String,
    manifest: ApiManifest,
    runtime: Runtime,
}

impl ApiRuntime {
    pub async fn new(
        api_path: String,
        root_mgr: RootManager,
        events: Rc<RefCell<Events>>,
    ) -> Result<Self> {
        // TODO(appcypher): Support permissions.
        let content = root_mgr.read_file_from_workspace(&format!("{}/{}", api_path, "api.yaml"))?;
        let manifest = ApiManifest::try_from(&content)?;

        // TODO(appcypher): Get permissions from config.
        let http_ev_allow_list = [HttpEventPath::from("/api/v1/*")];
        let fs_allow_list = [
            FsPath::from("/auth.js"),
            FsPath::from("/mine"),
            FsPath::from("/apps/frontend@v0.1")
        ];

        let permissions = Permissions::builder()
            .add_state(FsRoot::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../sample/workspaces/unreachable"
            )))
            .add_permissions(&[
                (HttpEvent::ReadRequest, &http_ev_allow_list),
                (HttpEvent::SendResponse, &http_ev_allow_list),
            ])?
            .add_permissions(&[(Fs::Open, &fs_allow_list), (Fs::Read, &fs_allow_list)])?
            .build();

        // Create runtime.
        let runtime = Runtime::with_events(permissions, events, true, Default::default()).await?;

        Ok(Self {
            api_path,
            root_mgr,
            manifest,
            runtime,
        })
    }

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

    async fn run_auth(&mut self) -> Result<bool> {
        // TODO(appcypher): Permissions.
        let filepath = "/auth.js";

        // Scripts are not modules so they all share scopes.
        // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
        let code = Self::format_code(&self.root_mgr.read_file_from_workspace(filepath)?);

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

    async fn run_middlewares(&mut self) -> Result<bool> {
        // TODO(appcypher):  Permissions.
        for path in self.manifest.middlewares.iter() {
            let filepath = &path.script;

            // Scripts are not modules so they all share scopes.
            // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
            let code = Self::format_code(&self.root_mgr.read_file_from_workspace(filepath)?);

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

    pub async fn run_index(&mut self) -> Result<()> {
        let filepath = &format!("{}/index.js", self.api_path);

        let code = &self.root_mgr.read_file_from_workspace(filepath)?;

        // Execute module.
        self.runtime.execute_module(filepath, code).await?;

        Ok(())
    }

    fn format_code(code: &str) -> String {
        // SEC: Note that there still ways to leak things into the global scope. https://gist.github.com/appcypher/2c210cd04774f1812a4b3e5c84496858
        // Not sure if this is a critical security issue yet.
        format!("\"use strict\"; (\n{} \n)();", code)
    }
}
