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
    events: Rc<RefCell<Events>>,
}

impl ApiRuntime {
    pub async fn new(file_mgr: FileManager, events: Rc<RefCell<Events>>) -> Result<Self> {
        // TODO(appcypher): Support permissions.
        let content = file_mgr.read_file_from_api_path("api.yaml").await?;
        let manifest = ApiManifest::try_from(&content)?;

        Ok(Self {
            file_mgr,
            manifest,
            events,
        })
    }

    pub async fn execute(&self) -> Result<bool> {
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

    async fn run_auth(&self) -> Result<bool> {
        // TODO(appcypher): Fs permissions.
        let filename = "/auth.js";
        let code = self.file_mgr.read_file(filename).await?;
        let events = Rc::clone(&self.events);

        // Execute module.
        let mut runtime = Runtime::default_event(Permissions::default(), events).await?;
        runtime.execute_module(filename, code).await?;

        Ok(true) // TODO(appcypher): Check value ok
    }

    async fn run_middlewares(&self) -> Result<bool> {
        // TODO(appcypher): Fs permissions.
        for path in self.manifest.middlewares.iter() {
            let filename = &path.script;
            let code = self.file_mgr.read_file(filename).await?;
            let events = Rc::clone(&self.events);

            // Execute module.
            let mut runtime = Runtime::default_event(Permissions::default(), events).await?;
            runtime.execute_module(filename, code).await?;

            // TODO(appcypher): Check value ok
        }

        Ok(true)
    }

    pub async fn run_index(&self) -> Result<()> {
        let filename = &self
            .file_mgr
            .paths
            .get_relative_path_from_api_path("index.js");
        let code = self.file_mgr.read_file(filename).await?;
        let events = Rc::clone(&self.events);

        // TODO(appcypher): Remove
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

        // Execute module.
        let mut runtime = Runtime::default_event(permissions, events).await?;
        runtime.execute_module(filename, code).await?;

        Ok(())
    }
}
