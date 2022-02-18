// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::permissions::Db;
use tera::{
    errors::AnyError,
    extensions::{op_async, op_sync, Extension, OpState, Resource, ResourceId},
    include_js_files,
    permissions::Permissions,
};

/*
    TODO(appcypher):
    There can be multiple databases.
    How do we connect to a database we don't know about yet?
    Is connecting to main by default the best option?
    We do not connect to any until necessary.

    // Extensions
    type DbPools = HashMap<String, (Pool, Vec<Conn>)>; // database name -> (connection pool -> connection)

    On Extension Init:
    - Connect to workspace `default` database.
    - Set ANSI mode.
        conn.query_drop("SET SESSION sql_mode = 'ANSI';")?;
    - Add connection to map.

    On Create Database:
    - Create new database.
        conn.query_drop("CREATE DATABASE IF NOT EXISTS db;")?;
    - Connect to new database.
    - Set ANSI mode.
        conn.query_drop("SET SESSION sql_mode = 'ANSI';")?;
    - Add connection to map.

    On Connect to Database:
    - Construct database url
    - Connect to database (the database name is optional in which case we don't connect to anything at all).
    - Set ANSI mode.
        conn.query_drop("SET SESSION sql_mode = 'ANSI';")?;
    - Add connection to map.
*/

pub fn db(permissions: Rc<RefCell<Permissions>>) -> Extension {
    // TODO(appcypher): Connect to workspace default database here. This serves as a starting point connection.

    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(runtime_server:extensions) ",
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

            Ok(())
        })
        .build();

    extension
}

// #[derive(Debug)]
// pub struct Databases(HashMap<String, (Pool, Vec<Conn>)>);

// impl Resource for Databases {}

fn op_db_connect(_state: &mut OpState, _db_name: String, _: ()) -> Result<ResourceId, AnyError> {
    // Check read permission.
    // let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    // permissions.borrow().check(DB::Connect, DB)?;

    Ok(0)
}

async fn op_db_query(
    _state: Rc<RefCell<OpState>>,
    _rid: ResourceId,
    _query: String,
) -> Result<String, AnyError> {
    // TODO(appcypher):
    // Parse SQL query to make sure we have permission to do any of it
    Ok(String::new())
}
