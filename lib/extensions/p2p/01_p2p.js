// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  async function p2pPeerConnect(key) {
    return core.opAsync("opP2pPeerConnect", key);
  }

  window.__bootstrap.httpEvent = {
    p2pPeerConnect,
  };
})(globalThis);
