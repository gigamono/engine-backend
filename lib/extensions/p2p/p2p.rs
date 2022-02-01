// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{cell::RefCell, rc::Rc};

use tera::{
    errors::AnyError,
    extensions::{Extension, OpState, ResourceId},
    include_js_files,
    permissions::Permissions,
};

pub fn p2p(permissions: Rc<RefCell<Permissions>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(backend:extensions) ",
            "lib/extensions/p2p/01_p2p.js",
        ))
        .ops(vec![
            // ("opP2pPeerConnect", op_async(op_p2p_peer_connect)),
        ])
        .state(move |state| {
            if !state.has::<Rc<RefCell<Permissions>>>() {
                state.put(Rc::clone(&permissions));
            }

            Ok(())
        })
        .build();

    extension
}

async fn op_p2p_peer_connect(
    _state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    _: (),
) -> Result<(), AnyError> {
    // TODO(appcypher): Add implementation.
    // Should open socket first, then use that rid to connect to peer. ???
    // Idempotent
    Ok(())
}
