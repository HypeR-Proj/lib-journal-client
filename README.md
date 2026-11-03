<!--
SPDX-FileCopyrightText: 2026 roolrz
SPDX-License-Identifier: Apache-2.0
-->

# lib-journal-client

A HypeR component. See [HypeR](https://github.com/HypeR-Proj/HypeR) for
workspace setup and the installed SDK build boundary.

## Workspace tests

Use the [HypeR workspace](https://github.com/HypeR-Proj/HypeR) to resolve dependencies:

```sh
python3 project.py test lib-journal-client
```

Paths in the documentation are relative to the repo-managed workspace root.
Edit this repository and prepare the workspace again before building.

## Query connection

After routing the `query` protocol, call `open_query(session, absolute_deadline)`
to receive the shared request endpoint. Pass the same caller deadline through
routing and endpoint transfer. The returned capability permits request, wait,
duplication and delegation; it grants no receive authority on the server side.
It can be retained or delegated to a shell without consuming a server session
slot. Submit `Query` values with `hyper_service_ipc::request`; each uses a private
reply channel. Incomplete endpoint handoffs expire after five seconds.
