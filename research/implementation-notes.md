# Implementation notes for later work

Checked 2026-10-16. No prototype, build, or network test has been performed.

## Version baseline

| Item | Observed version | Evidence and qualification |
| --- | --- | --- |
| Core `iroh` | 1.3.0 | GitHub latest release and crates.io agree; released 2026-09-28, not yanked |
| Core minimum Rust | 1.91 | crates.io metadata and Cargo manifest |
| `@number0/iroh` | 1.1.0 | npm `latest`; requires Node >=20.3.0 |
| `iroh-ping` | 1.0.0 | Repository main Cargo manifest; depends on Iroh 1.0.0 semver range |
| `iroh-blobs` | 0.103.1 | Repository main Cargo manifest, not independently verified as latest published crate |
| `iroh-gossip` | 0.101.0 | Repository main Cargo manifest, not independently verified as latest published crate |
| `iroh-docs` | 0.101.0 | Repository main Cargo manifest, not independently verified as latest published crate |
| `iroh-ffi` | 1.1.0 | Repository main Cargo manifest |

The inspected companion manifests all target Iroh 1.x. Their version numbers
need not match the core number. A manifest inspection does not prove that a
particular combined application builds. Before coding, verify published versions,
select compatible protocol crates/bindings, and commit the application's lockfile.

Sources: [release](https://github.com/n0-computer/iroh/releases/tag/v1.3.0),
[crates.io metadata](https://crates.io/api/v1/crates/iroh),
[npm metadata](https://registry.npmjs.org/@number0%2firoh/latest), and manifest
URLs in [the source register](sources.json).

## API landmarks verified against the 1.3.0 tag

The following are orientation snippets, not a complete compiled application:

```rust
use iroh::{Endpoint, endpoint::presets, protocol::Router};

let endpoint = Endpoint::bind(presets::N0).await?;
// Or: Endpoint::builder(presets::N0).secret_key(saved_key).bind().await?;
let id = endpoint.id();
let addr = endpoint.addr();

// Server: register a ProtocolHandler under an application-specific ALPN.
let router = Router::builder(endpoint)
    .accept(ALPN, handler)
    .spawn();
// Keep router alive for the serving lifetime, then:
router.shutdown().await?;
```

The connecting side uses `endpoint.connect(addr, ALPN).await?`, then `open_bi()`
or other QUIC operations. The receiving side can use `accept_bi()` in its
handler. `Connection::remote_id()` exposes the authenticated peer identity.
A raw endpoint can also use an explicit accept loop instead of a router.

In the pinned echo example, `ProtocolHandler::accept` returns
`Result<(), AcceptError>`. `Router::spawn()` is synchronous in that example;
it is not followed by `.await?`. Match the trait and error types to our exact
version rather than copying a snippet from a different generation.

Sources: [endpoint source](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/src/endpoint.rs),
[presets](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/src/endpoint/presets.rs),
[echo](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/examples/echo.rs).

## Implementation details worth remembering

- **Readiness:** `bind()` does not mean a relay is ready. `online().await` waits
  for a relay handshake, has no built-in timeout, and can wait forever with no
  relay or no WAN connectivity. It does not wait for address-lookup publication.
  Put a deadline around internet readiness; do not gate LAN-only startup on it.
- **Identity:** persist a secret key for a stable peer. Handle a missing key
  separately from corrupt data or permission failures; silently replacing an
  unreadable key creates a new identity. Protect the key and avoid running two
  devices with the same key. Endpoint ID is not automatically a user account.
- **Peer addressing:** use fresh endpoint addresses/tickets for immediate pairing.
  With working lookup, persist the peer ID and resolve current reachability.
  A ticket's stale addresses need not make an ID unusable, but lookup must exist.
- **Default infrastructure:** `presets::N0` installs Pkarr publication/resolution,
  native DNS lookup, default relays, and a crypto provider. A private deployment
  needs an explicit decision about both lookup and relay infrastructure.
- **Stream framing:** QUIC streams are byte streams. Add bounded length framing
  for repeated messages. A peer cannot accept a newly opened stream until the
  initiator sends data. Bound reads, concurrent work, and timeouts.
- **Graceful shutdown:** finish the send stream, let the recipient consume the
  final data, then close the connection/endpoint or shut down the router. The
  pinned echo example waits for the remote to close after sending its response.
- **Authorization:** endpoint hooks can reject a connection after authenticated
  handshake. Protocol handlers still need operation-level policy where required.
  An ALPN identifies a protocol; it is not an access credential.
- **Path evidence:** inspect connection paths/events and statistics; successful
  exchange alone does not prove a direct connection. Paths may change during a
  session. The hooks guide references `path_events()`, `paths()`, and weak handles.
- **Large files:** the blobs tutorial uses `MemStore` and can hold the file in RAM.
  Evaluate `FsStore`, export behavior, retained partial data, and tags/garbage
  collection before claiming durable resumable transfers.
- **Replicated state:** persist both docs metadata and blobs. Handle metadata
  insertion and `ContentReady` separately. Define the intended semantics for
  concurrent writes and deletion before choosing a query/model.

Sources: pinned source above; [hooks](https://docs.iroh.computer/connecting/endpoint-hooks),
[blobs](https://docs.iroh.computer/protocols/blobs),
[documents](https://docs.iroh.computer/protocols/documents).

## Browser and binding choices

Rust is the most direct reference implementation. Node.js can use native N-API
bindings via `@number0/iroh`; Python, Swift, Kotlin, Go, and C have binding options.
Confirm actual platform coverage and which higher-level protocols are exposed;
core connectivity bindings do not imply that all Rust ecosystem crates have bindings.
For Python, the guide requires registering the running asyncio loop with
`iroh.iroh_ffi.uniffi_set_event_loop(...)` before using the binding APIs;
otherwise callbacks from Rust threads can fail or hang.

For browsers, use Rust/Wasm integration examples. Relay-only transport affects
cost and performance. The Wasm guide recommends disabling default crate features;
however, the 1.3.0 `N0` preset is gated on a crypto-provider feature (`tls-ring`
or `tls-aws-lc-rs`). Check the exact browser example's dependency features and
crypto-provider setup when implementing; do not assume that dropping every
default feature leaves all native snippets usable.

A native process plus local UI is a plausible prototype architecture if we want
a familiar UI and native networking. The UI/backend boundary and packaging still
need to be selected together.

Sources: [JavaScript](https://docs.iroh.computer/languages/javascript),
[Python](https://docs.iroh.computer/languages/python),
[Wasm](https://docs.iroh.computer/languages/wasm-browser), pinned presets source.

## Network configuration and deployment

The enterprise networking guide describes relay connections over HTTPS upgraded
to WebSocket on TCP 443. Public address publication uses HTTPS; native lookup can
use system DNS. Direct connections require outbound UDP to peer-selected ports,
not only UDP 443. The guide also describes UDP 7842 for address discovery.
Web browsing working through a proxy does not prove relay WebSocket support.

Port mapping (UPnP/NAT-PMP/PCP) can be enabled in the normal native build; mDNS
is opt-in. Inspect actual configuration before making claims such as “no router
ports are opened.” The current API provides `clear_ip_transports()` to remove
IP transports; the networking guide combines that with disabled port mapping
for a relay-only configuration. That is a useful controlled fallback test.

The self-hosting guide identifies itself as community-maintained and potentially
out of date. Use the selected release's relay README/configuration source before
deployment.

Public relay operation is subject to change and version support is not a long-term
compatibility guarantee. A self-hosted relay does not automatically eliminate
public lookup. Managed-service pricing and limits should be checked when needed;
we have not subscribed to any service or produced a cost estimate.

Source: [network configuration](https://docs.iroh.computer/configuring-networks),
[public relays](https://docs.iroh.computer/iroh-services/relays/public),
[release policy](https://docs.iroh.computer/about/release-policy).

## Documentation contradictions found during research

Prefer version-pinned code and API signatures when tutorial snippets disagree.

| Location | Observation | Working conclusion |
| --- | --- | --- |
| `www.iroh.computer/llms.txt` | Uses older `NodeId` / `NodeAddr` terminology and legacy docs paths | Start from `docs.iroh.computer/llms.txt`; current types are `EndpointId` / `EndpointAddr` |
| Core repository main README | Shows no-argument `Endpoint::bind()` and awaited router spawn | Use the 1.3.0 echo example and signatures instead |
| Creating-an-endpoint guide, persistent identity snippet | Shows `Endpoint::builder()` then `.bind(presets::N0)` | 1.3.0 signature is `Endpoint::builder(presets::N0)` then `.bind()` |
| Security page | Calls relay-only mode “upcoming,” but later says direct connections can be disabled | 1.3.0 source contains `clear_ip_transports()`; networking guide documents its use |
| FAQ, WebRTC comparison | Both acknowledges browser support and later says Iroh does not run in browsers | Dedicated Wasm guide explicitly supports browsers with relay-only traffic |
| Blobs tutorial | Download snippet has unbalanced parentheses | Do not copy literally; consult the selected crate's complete transfer example |
| Gossip guide | Suggests public rendezvous/ticket broadcasting without a concrete implementation in the inspected material | Treat managed topic discovery as unverified; supply explicit bootstrap peers |
| Public-relay support policy | One page says latest stable release, another latest major series | Assume no guaranteed support for older versions; recheck before deployment |

These are observations from the fetched pages on 2026-10-16, not claims that
upstream examples are permanently broken. Source URLs and fetched-content hashes
are recorded in sources.json so the research basis is identifiable.

## Local environment and verification boundary

At research time this execution environment had Node.js **22.22.1**. Neither
`cargo` nor `rustc` was found on its PATH. This says nothing about software
installed elsewhere on the host. No toolchain or project dependencies were
installed, and no public relay connections were initiated as an application.

Performed: HTTP retrieval of official docs; inspection of repository examples,
Cargo manifests, version-pinned core source, GitHub release metadata, crates.io,
and npm metadata. Local Markdown links and repository setup were checked.

Not performed: compilation, execution of examples, NAT traversal tests,
throughput measurements, security audit, or binding interoperability tests.
