# Local API v1 and SDK

Default origin `http://127.0.0.1:7331`. Bind is restricted to loopback; exact Host
and any Origin must match configured API. No permissive CORS or privileged Vite
proxy exists. `/health` is public, nonsensitive readiness metadata. All `/v1/`
paths require `Authorization: Bearer TOKEN`. The static built UI is same-origin.

| Endpoint | Result |
|---|---|
| GET `/health` | READY, protocol/candidate versions |
| GET `/v1/status` | Operator snapshot of actual local state |
| GET `/v1/identity`, `/peers`, `/network` | Public device key, pinned trust and current paths |
| GET `/v1/resources`, `/models` | Signed selected advertisement and provider records |
| GET `/v1/jobs`, `/grants`, `/storage`, `/messages`, `/ledger` | Bounded operator detail views |
| GET `/v1/doctor` | Key/clock/disk and full surviving ledger checks |
| GET `/v1/events` | Authenticated committed ledger SSE with GAP notification |
| POST `/v1/control` | `{ "operation": "job.run", "args": { ... } }` |

Finite operations: peer.approve/revoke/connect/rotate/request; porch.create/
invite/join/leave; grant.issue/import/revoke; resources.refresh; model.discover;
share.configure; job.run; storage.put/get/replicate/fetch/pin/delete/delete_remote;
message.send/channel/retry; address.resolve; client.issue/revoke. relay.enable,
federation.enable and compute.wasm explicitly refuse ENGINE_NOT_INSTALLED.
Advanced peer.request allows only bounded existing storage/message/resource
operations and still requires trust/provider authority; it cannot send raw jobs.

```json
{
  "operation": "job.run",
  "args": {
    "id": "application-request-001",
    "capability": "compute.hash",
    "resource": "sha256",
    "input": "abc",
    "privacy": "LOCAL_ONLY",
    "max_output_tokens": 16,
    "timeout_ms": 1000
  }
}
```

Result includes status, explainable route and signed receipt for an executor
decision. Pre-routing refusal may have only a reason/route and local event
evidence; malformed/missing input returns HTTP 400. Authentication failures use
401, forbidden origin/scope 403, oversized delegated requests 413. Clients must
inspect both HTTP status and returned execution status.

`client.issue` requires operator authority. Its credential binds holder label,
token digest, expiry, max calls, read scopes, operations, exact resources and
privacy classes. Allowed reads: models/peers/resources/network; operations:
job.run/message.send/resources.refresh. It cannot read private operator records
or change authority. New requests consume its budget transactionally. Defaults
are LOCAL_ONLY and job.run, with no resources until explicitly listed.

```ts
import { PorchClient } from '@porch/sdk';
const client = new PorchClient(scopedToken, 'http://127.0.0.1:7331');
const result = await client.run({capability:'compute.hash',resource:'sha256',input:'abc'});
```

See the exported SDK types/adapters in packages/sdk/src/index.ts; applications
can consume its compiled package or source workspace. `stream()` uses authenticated
fetch/SSE and a bounded parser. Do not persist the operator credential in browser
storage. The SDK prevents external API endpoints/path injection and defaults
to local processing; the daemon independently enforces authority.
