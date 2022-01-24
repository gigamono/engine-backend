// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{convert::TryFrom, path::Path};

use tera::permissions::{
    events::event_http::HttpEvent,
    fs::{Fs, FsPath, FsRoot},
    PermissionType, Permissions, Resource,
};
use utilities::{config::ApiManifest, result::Result};

type PermissionTuple = (Box<dyn PermissionType>, Vec<Box<dyn Resource>>);

pub struct ApiPermissions;

impl ApiPermissions {
    pub fn load_permissions(
        api_manifest: &ApiManifest,
        workspace_path: &Path,
    ) -> Result<Permissions> {
        let fs_permissions = Self::fs_permissions(api_manifest);
        let http_event_permissions = Self::http_event_permissions(api_manifest);

        Ok(Permissions::builder()
            .add_state(FsRoot::try_from(workspace_path)?)
            .add_owned_permissions(http_event_permissions)?
            .add_owned_permissions_with_allow_lists(fs_permissions)?
            .build())
    }

    fn fs_permissions(api_manifest: &ApiManifest) -> Vec<PermissionTuple> {
        if let Some(permissions) = &api_manifest.permissions {
            let mut result: Vec<PermissionTuple> = vec![];

            Self::add_permission_if_exists(&permissions.fs.open, Fs::Open.into(), &mut result);
            Self::add_permission_if_exists(&permissions.fs.create, Fs::Create.into(), &mut result);
            Self::add_permission_if_exists(&permissions.fs.read, Fs::Read.into(), &mut result);
            Self::add_permission_if_exists(&permissions.fs.write, Fs::Write.into(), &mut result);
            Self::add_permission_if_exists(
                &permissions.fs.execute,
                Fs::Execute.into(),
                &mut result,
            );

            return result;
        };

        vec![]
    }

    fn http_event_permissions(api_manifest: &ApiManifest) -> Vec<Box<dyn PermissionType>> {
        if let Some(permissions) = &api_manifest.permissions {
            let mut result: Vec<Box<dyn PermissionType>> = vec![];

            if permissions.http_event.request_read {
                result.push(HttpEvent::RequestRead.into())
            }

            if permissions.http_event.response_send {
                result.push(HttpEvent::ResponseSend.into())
            }

            return result;
        }

        vec![]
    }

    fn add_permission_if_exists(
        list: &[String],
        permission_type: Box<dyn PermissionType>,
        result: &mut Vec<PermissionTuple>,
    ) {
        if list.len() > 0 {
            result.push((
                permission_type,
                list.iter().map(|s| FsPath::from(s).into()).collect(),
            ))
        }
    }
}
