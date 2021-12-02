// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::rc::Rc;

use crate::files::FileManager;
use tera::{events::Events, Runtime};
use utilities::{config::SurlManifest, result::Result};

pub struct SurlRuntime {
    file_mgr: FileManager,
    manifest: SurlManifest,
    events: Rc<Events>,
}

impl SurlRuntime {
    pub async fn new(file_mgr: FileManager, events: Rc<Events>) -> Result<Self> {
        // Save Surl manifest.
        let content = file_mgr.read_file_from_surl("surl.yaml").await?;
        let manifest = SurlManifest::new(&content)?;

        Ok(Self {
            file_mgr,
            manifest,
            events,
        })
    }

    pub async fn execute(&self) -> Result<bool> {
        // Run auth.
        if !self.run_auth().await? {
            return Ok(false);
        };

        // Run middlewares
        if !self.run_middlewares().await? {
            return Ok(false);
        };

        // Run index.
        self.run_index().await?;

        Ok(true)
    }

    async fn run_auth(&self) -> Result<bool> {
        let filename = "/system/auth.js";
        let code = self.file_mgr.read_file(filename).await?;
        let events = Rc::clone(&self.events);

        // Execute module.
        let permissions = Default::default();
        let mut runtime = Runtime::default_event(permissions, events).await?;
        runtime.execute_module(filename, code).await?;

        Ok(true) // TODO(appcypher): Check value ok
    }

    async fn run_middlewares(&self) -> Result<bool> {
        for path in self.manifest.middlewares.iter() {
            let filename = format!("/middlewares/{}", path);
            let code = self.file_mgr.read_file(&filename).await?;
            let events = Rc::clone(&self.events);

            // Execute module.
            let permissions = Default::default();
            let mut runtime = Runtime::default_event(permissions, events).await?;
            runtime.execute_module(filename, code).await?;

            // TODO(appcypher): Check value ok
        }

        Ok(true)
    }

    pub async fn run_index(&self) -> Result<()> {
        let filename = &self.file_mgr.paths.get_relative_path_from_surl("index.js");
        let code = self.file_mgr.read_file(filename).await?;
        let events = Rc::clone(&self.events);

        // Execute module.
        let permissions = Default::default();
        let mut runtime = Runtime::default_event(permissions, events).await?;
        runtime.execute_module(filename, code).await?;

        Ok(())
    }
}
