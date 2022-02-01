// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::permissions::{PermissionType, PermissionTypeKey};
use std::any::TypeId;

#[derive(Debug, Clone)]
pub enum DB {
    DatabaseConnect,
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

// TODO(appcypher):
// We use an ASCII-based addressing scheme similar to a url.

impl PermissionType for DB {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }
}
