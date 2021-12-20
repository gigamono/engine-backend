// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::{cell::RefCell, rc::Rc};

use crate::files::FileManager;
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
    file_mgr: FileManager,
    manifest: ApiManifest,
    runtime: Runtime,
}

impl ApiRuntime {
    pub async fn new(file_mgr: FileManager, events: Rc<RefCell<Events>>) -> Result<Self> {
        // TODO(appcypher): Support permissions.
        let content = file_mgr.read_file_from_api_path("api.yaml").await?;
        let manifest = ApiManifest::try_from(&content)?;

        // TODO(appcypher): Get permissions from config.
        let http_ev_allow_list = [HttpEventPath::from("/api/v1/*")];
        let fs_allow_list = [FsPath::from("/auth.js"), FsPath::from("/mine")];

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
        let runtime = Runtime::with_events(permissions, events, Default::default()).await?;

        Ok(Self {
            file_mgr,
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
        let filename = "/auth.js";

        // Scripts are not modules so they all share scopes.
        // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
        let code = format!(
            "\"use strict\"; (function main(){{ \n{}\n }})();",
            self.file_mgr.read_file(filename).await?
        );

        // TODO(appcypher): Get permissions from config.
        let permissions = Permissions::default();

        // Execute script.
        let value_global = self
            .runtime
            .execute_middleware_script(filename, code, permissions)
            .await?;

        let scope = &mut self.runtime.handle_scope();
        let value = value_global.open(scope);

        Ok(value.boolean_value(scope))
    }

    async fn run_middlewares(&mut self) -> Result<bool> {
        // TODO(appcypher):  Permissions.
        for path in self.manifest.middlewares.iter() {
            let filename = &path.script;

            // Scripts are not modules so they all share scopes.
            // The template around the code is to make sure they run synchronously and to prevent namespace pollution.
            let code = format!(
                "\"use strict\"; (function main(){{ \n{}\n }})();",
                self.file_mgr.read_file(filename).await?
            );

            // TODO(appcypher): Get permissions from config.
            let permissions = Permissions::default();

            // Execute script.
            let value_global = self
                .runtime
                .execute_middleware_script(filename, code, permissions)
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
        let filename = &self
            .file_mgr
            .paths
            .get_relative_path_from_api_path("index.js");

        let code = self.file_mgr.read_file(filename).await?;

        // Execute module.
        self.runtime.execute_module(filename, code).await?;

        Ok(())
    }
}
