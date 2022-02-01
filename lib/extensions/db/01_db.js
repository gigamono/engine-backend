// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function dbConnect(dbName) {
    return core.opSync("opDbConnect", dbName);
  }

  async function dbQuery(rid, query) {
    return core.opAsync("opDbQuery", rid, query);
  }

  window.__bootstrap.httpEvent = {
    dbConnect,
    dbQuery,
  };
})(globalThis);
