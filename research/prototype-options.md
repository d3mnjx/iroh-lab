# Prototype discussion notes

Prepared 2026-10-16. **No option has been selected.** The user plans to ask about
Iroh and design the prototype together before implementation.

## Candidate experiments

| Experiment | What we would build | What it would teach us | Additional complexity |
| --- | --- | --- | --- |
| Connection inspector | Two native peers exchange messages and show identity, connection time, RTT, and observed paths | Whether Iroh connects our actual devices directly, and how fallback behaves | Smallest scope; a diagnostic rather than a full product |
| File handoff | Sender shares a ticket; receiver downloads with progress and integrity verification | File transfer, interruptions, relay throughput, and sender availability | Blob storage, export paths, retention and access rules |
| Small group chat | Join by invitation; exchange live messages through gossip | Group membership, fan-out and behavior under churn | History and offline delivery are separate work |
| Shared todo/notes state | Two peers edit local data and reconcile after reconnecting | Local-first persistence, convergence and conflict semantics | More demanding data model, permissions, and storage lifecycle |
| Remote device control | Paired peers exchange bounded commands and responses | Request/response workflows without port forwarding | Pairing, authorization and command safety matter immediately |

Recommendation for discussion: use a connection inspector as the first technical
checkpoint beneath whichever product idea we choose. If the desired experience
is already clear, make that checkpoint part of it rather than building a separate UI.

## Decisions to make together

1. What real task should the demo perform: move a file, send messages, sync state,
   or control a device?
2. Which two devices and networks will we actually test? Native desktop, phone,
   browser, and server imply different integration and direct-connection limits.
3. Is installing an app/CLI acceptable, or must a recipient use only a link?
4. Must both peers be online at once, or should data remain available later?
5. Is this a trusted-device demo, an invitation-only group, or public sharing?
6. What observation would make this experiment worthwhile: successful NAT traversal,
   simpler pairing, speed, offline sync, or a better user workflow?

These questions are retained for the next conversation; they are not blockers
for the completed research and are not requests to provision infrastructure now.

## Validation plan once we choose a prototype

| Scenario | Evidence to collect | What it does not establish |
| --- | --- | --- |
| Two processes on one machine | Basic protocol correctness and shutdown | WAN NAT traversal or realistic latency |
| Two devices on the same LAN | Successful exchange, actual path, local RTT | Connectivity across different routers |
| Home network to mobile hotspot/another network | Connect success/failure, time to connect, paths over time, RTT | Universal connectivity from a single successful trial |
| Native endpoint deliberately configured for relay-only | Functional fallback, path evidence, relay throughput | Real firewall behavior unless separately tested |
| Temporarily unavailable peer or interrupted connection | Bounded failure, useful UI, retry behavior | Automatic offline delivery |
| Restart with a saved identity | Stable endpoint ID and reconnection | Data persistence unless storage is also exercised |
| Browser peer, if in scope | Interoperability and observed relayed path | Native-style direct browser connectivity |
| Uninvited peer, if access is restricted | Rejection before protected application operations | A complete security review |

For file transfer: compare verified content, interrupt and resume, account for
persistent partial state, try a file larger than trivial memory buffers, and
record what happens when the provider exits. For replicated state: make edits
while disconnected, reconnect, inspect converged values and content readiness,
and restart both peers to validate persistence.

Record exact software versions, OS, network topology, transport settings,
payload sizes, failures, and path transitions. Use synthetic data initially.
Set success criteria after choosing devices and workload; do not invent a
throughput target before measuring a baseline.

## Existing starting points

- [iroh-ping](https://github.com/n0-computer/iroh-ping): current official quickstart.
- [Pinned echo example](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/examples/echo.rs): custom bidirectional stream protocol.
- [sendme](https://github.com/n0-computer/sendme): working file/directory transfer tool to evaluate before rebuilding its essentials.
- [browser-echo](https://github.com/n0-computer/iroh-examples/tree/main/browser-echo)
  and [browser-chat](https://github.com/n0-computer/iroh-examples/tree/main/browser-chat): Wasm integration references.
- [tauri-todos](https://github.com/n0-computer/iroh-examples/tree/main/tauri-todos): desktop replicated-state example.
- [hello-iroh-ffi](https://github.com/n0-computer/hello-iroh-ffi): cross-platform presence demo.

The ping source and sendme README were inspected. The other example directories
are references to evaluate, not projects built or tested in this research session.
