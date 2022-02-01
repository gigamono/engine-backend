// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use mysql::{Conn, Pool};
use tera::{
    errors::AnyError,
    extensions::{op_async, op_sync, Extension, OpState, Resource, ResourceId},
    include_js_files,
    permissions::Permissions,
};

pub fn db(permissions: Rc<RefCell<Permissions>>, db_pool: Rc<Pool>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(backend:extensions) ",
            "lib/extensions/db/01_db.js",
        ))
        .ops(vec![
            ("opDbConnect", op_sync(op_db_connect)),
            ("opDbQuery", op_async(op_db_query)),
        ])
        .state(move |state| {
            if !state.has::<Rc<RefCell<Permissions>>>() {
                state.put(Rc::clone(&permissions));
            }

            if !state.has::<Rc<Pool>>() {
                state.put(Rc::clone(&db_pool));
            }

            Ok(())
        })
        .build();

    extension
}

#[derive(Debug)]
pub struct Databases(HashMap<String, Conn>);

impl Resource for Databases {}

fn op_db_connect(_state: &mut OpState, _db_name: String, _: ()) -> Result<ResourceId, AnyError> {
    // TODO(appcypher): Add implementation.
    // This function must be idempotent.
    Ok(0)
}

async fn op_db_query(
    _state: Rc<RefCell<OpState>>,
    _rid: ResourceId,
    _query: String,
) -> Result<String, AnyError> {
    // TODO(appcypher): Add implementation
    Ok(String::new())
}
