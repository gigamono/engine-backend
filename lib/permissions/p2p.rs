// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use tera::permissions::{PermissionType, PermissionTypeKey};
use std::any::TypeId;

#[derive(Debug, Clone)]
pub enum P2P {
    SocketOpen,
    SocketClose,
    PeerConnect,
    PeerDisconnect,
    PeerSend,
    PeerRecieve,
}

impl PermissionType for P2P {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }
}

