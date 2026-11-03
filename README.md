<!--
SPDX-FileCopyrightText: 2026 roolrz
SPDX-License-Identifier: Apache-2.0
-->

# lib-journal-client

Journal registration and query contracts for Native clients.

Supervisors register sources and transfer byte writers to services. Query helpers
negotiate private request endpoints; records include supervisor-assigned component
identity, generation and stream metadata.

See the [implementation entry](src/lib.rs) and [component declaration](component.json)
for the public interfaces, build dependencies and delivered artifacts.

## Boundaries

Ordinary log bytes cannot impersonate another source. Registration and query frame
bounds are part of the shared protocol; callers must handle rejection, expiry and
retention loss.

## Build and test

Use this checkout inside the [repo-managed workspace](https://github.com/HypeR-Proj/HypeR).
From this repository directory:

```sh
make
make test
```

An absolute `OUT=/path/to/out` selects a shared output directory. The module
entry delegates to build/system, which prepares the installed SDK and declared
dependencies. Native binaries require HypeR; host module checks cover the tests
selected by component.json. A standalone clone does not contain the workspace’s
other source repositories or Native SDK.


## Query connection

After routing the `query` protocol, call `open_query(session, absolute_deadline)`
to receive the shared request endpoint. Pass the same caller deadline through
routing and endpoint transfer. The returned capability permits request, wait,
duplication and delegation; it grants no receive authority on the server side.
It can be retained or delegated to a shell without consuming a server session
slot. Submit `Query` values with `hyper_service_ipc::request`; each uses a private
reply channel. Incomplete endpoint handoffs expire after five seconds.

## Source registration

`registration` owns the client/server wire validation. A component name is
1–255 UTF-8 bytes and contains no Unicode control characters. The complete
registration, including its four-byte prefix and JSON escaping, must fit in
512 bytes. `register` rejects invalid local input with `INVALID_ARGUMENT` before
sending or transferring a capability. The pure encoder/decoder reports the
specific validation error. Journald uses the same decoder before admitting a
source; raw `LOGS` names follow the same character and name-size rules.
