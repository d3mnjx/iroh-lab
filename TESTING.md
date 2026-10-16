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

- [Build/deployment run](https://github.com/d3mnjx/iroh-lab/actions/runs/38905175667)
  completed successfully for application commit `0729d67`.
- https://d3mnjx.github.io/iroh-lab/ and its manifest returned HTTP 200 over HTTPS.
- The same **3 real browser integration tests passed against the published URL**:
  messaging in both directions and measured ping, reconnect, invalid invitation
  rejection, manifest/Wasm caching, and offline reopening.
- The selected path reported by Iroh was `relay`. A local sample application
  round trip measured 33 ms; this is an observation, not a performance promise.
- [Follow-up deployment](https://github.com/d3mnjx/iroh-lab/actions/runs/38905833349)
  completed successfully for application commit `eef14ab`. Its published-page
  test passed for rejecting a bad invitation and accepting a valid replacement
  in the same open tab without reloading.
- Restart validation confirmed the same endpoint ID, a changed token, rejection
  of the previous invitation, and a successful fresh connection from Pages.
- A fresh native process was prepared after testing so the user's phone session
  starts with clean temporary message history. Its invitation remains local.

Documentation-only commits after the application commit do not trigger a rebuild;
the Pages workflow ignores Markdown and research-only changes.

## Still requires the user's phone

- Real iOS/Android install flow and installed standalone behavior.
- Connectivity from the user's actual Wi-Fi/mobile carrier.
- Phone lock/background suspension and resume behavior.

No universal throughput, direct-connect success rate, uptime, or production
security guarantees are inferred. Ping is a current application round trip,
not a benchmark. The cached shell works offline; messaging does not.

## iOS connection timeout fix (2026-10-16)

The user reported an iOS 26 / Wi-Fi timeout. Linux WebKit 26.5 reproduced the
published app's timeout and logged `Unacceptable TLS certificate` for
`wss://euc1-1.relay.n0.iroh.link./relay`. The same WebKit instance successfully
opened a verified secure WebSocket to the hostname without its terminal dot.

The browser now normalizes the invitation relay's hostname and uses that relay
for both its endpoint configuration and the remote address. The minimal preset
avoids falling back to Iroh's dotted default relay/probe URLs; explicit pairing
already supplies the peer identity and relay. TLS certificate validation remains
enabled. The native peer and existing invitation token were left running.

Executed locally after the change:

- 6 Rust unit tests passed, including URL normalization and HTTPS enforcement.
- All 3 Chromium integration tests passed.
- WebKit pairing, bidirectional messages, ping, reconnect, invalid-token rejection,
  and replacement invitation flows passed.
- The WebKit PWA test verified manifest/cache setup, then failed with the browser's
  `WebKit encountered an internal error` during emulated-offline page reload.
  Offline reopening on physical iOS remains unverified; this is separate from
  the reproduced relay TLS failure and is not reported as a passing test.

Regression checks verify that the browser actually opens relay WebSockets without
terminal-dot hostnames and emits no corresponding TLS-certificate errors.
Connection errors now appear next to the pairing controls, not below the chat.

Published validation: [deployment run 38915484869](https://github.com/d3mnjx/iroh-lab/actions/runs/38915484869)
succeeded for commit `35f8692`. Both WebKit connection tests then passed against
https://d3mnjx.github.io/iroh-lab/, including real message exchange and replacement
invitations. The original user invitation is still valid; the native peer was
not restarted for this fix. Existing cached app tabs must close before the new
service worker takes over.

## Interactive architecture diagram

`web/diagram.html` is a standalone HTML illustration with five selectable flows.
Chromium and WebKit checks passed for flow selection, node explanations, animation
pause, mobile width, and navigation under the service worker. Chromium also
reopened both the diagram and app shell offline. Service-worker navigation now
serves known HTML assets individually instead of returning the app shell for the
diagram URL. The diagram contains no live credentials or message data.

The user subsequently confirmed the real iPhone connected, and the sandbox CLI
received their `hello`. Phone installation and background behavior remain separate
checks; the diagram does not imply automatic delivery into the assistant chat.
