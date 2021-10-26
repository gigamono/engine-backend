use crate::file_manager::FileManager;
use tera::runtime::{Global, Runtime as TeraRuntime, Script, Value};
use utilities::{config::SurlManifest, result::Result};

pub(crate) struct SurlExecutor {
    file_mgr: FileManager,
    manifest: SurlManifest,
}

impl SurlExecutor {
    pub async fn new(file_mgr: FileManager) -> Result<Self> {
        let content = file_mgr.read_file_from_surl("surl.yaml").await?;
        let manifest = SurlManifest::new(&content)?;
        Ok(Self { file_mgr, manifest })
    }

    pub async fn execute(&self) -> Result<Option<Global<Value>>> {
        // Run auth script.
        if !self.run_auth_script().await? {
            return Ok(None);
        };

        // Run middleware script
        if !self.run_middleware_scripts().await? {
            return Ok(None);
        };

        // Run index script.
        Ok(Some(self.run_index_script().await?)) // TODO: HandlerResult
    }

    async fn run_auth_script(&self) -> Result<bool> {
        // Form script.
        let filename = "/system/auth.js";
        let code = &self.file_mgr.read_file(filename).await?;

        // Run script.
        let result = TeraRuntime::new().execute(&Script::new(filename, code))?;

        // result.get(scope);

        Ok(true) // TODO: Check value ok
    }

    async fn run_middleware_scripts(&self) -> Result<bool> {
        for path in self.manifest.middleware_scripts.iter() {
            let filename = &format!("/middlewares/{}", path);
            let code = &self.file_mgr.read_file(&filename).await?;

            // Run script.
            let result = TeraRuntime::new().execute(&Script::new(filename, code))?;

            // TODO: Check value ok
        }

        Ok(true)
    }

    pub async fn run_index_script(&self) -> Result<Global<Value>> {
        // Form script.
        let filename = &self.file_mgr.paths.get_relative_path_from_surl("index.js");
        let code = &self.file_mgr.read_file(filename).await?;

        // Run script.
        TeraRuntime::new().execute(&Script::new(filename, code))
    }
}
