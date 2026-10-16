# Validation record

Date: 2026-10-16. This records executed checks, distinct from the earlier research.

## Environment

- Linux x86_64 sandbox; Rust 1.95.0; Iroh 1.3.0; wasm-bindgen 0.2.122.
- Node.js 22.22.1; Playwright 1.61.1 with installed Chromium 156.0.8078.4.
- Mobile browser viewport plus a desktop layout inspection. This is browser
  emulation, not a physical phone or cellular-network test.
- Native peer connected to a public Number 0 relay in the EU region.
- Browser and native application traffic passed through the public relay;
  connectivity was real, not mocked. The UI reads the selected Iroh path.

## Executed local checks

| Check | Outcome |
| --- | --- |
| Rust protocol and native-peer unit tests | 5 passed: message byte limits, request parsing, invitation validation, authorization before mutation, history eviction/cursors |
| JavaScript invitation parsing tests | 2 passed: full-link/fragment/raw invitations; malformed, oversized, unsupported and insecure inputs |
| Rust native build | Passed |
| Rust/WebAssembly release build | Passed; Wasm approximately 4 MiB |
| Formatting | Passed |
| Native Clippy | Passed after disabling incremental caches on the host mount |
| Browser pairing, messages both ways, ping, disconnect/reconnect | Passed through the real Iroh relay |
| Browser authorization failure | Invalid token rejected with a readable error |
| HTML-like message content | Displayed as literal text; no HTML element injection |
| PWA manifest and offline shell | Standalone manifest and Wasm cache verified; app reopened with browser networking disabled |
| Private-link handling | Invitation fragment removed from address bar; no localStorage persistence |
| Mobile layout | No horizontal overflow; 390 px layout visually inspected |
| Native restart | Endpoint ID persisted; new process generated a new invitation token |

The first request parser test exposed a serde flatten/unknown-field conflict,
which was fixed before successful network tests. Initial browser test launches
also caught a local process-start issue; the detached launcher now waits for an
actual ready peer. These failures were setup/development findings, not successful
connectivity evidence.

## Hosted verification

The GitHub Pages deployment and tests against its public HTTPS URL are recorded
below once deployment finishes.

## Still requires the user's phone

- Real iOS/Android install flow and installed standalone behavior.
- Connectivity from the user's actual Wi-Fi/mobile carrier.
- Phone lock/background suspension and resume behavior.

No universal throughput, direct-connect success rate, uptime, or production
security guarantees are inferred. Ping is a current application round trip,
not a benchmark. The cached shell works offline; messaging does not.
