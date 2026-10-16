# Source register

Accessed **2026-10-16 UTC**. Prefer the official documentation at
https://docs.iroh.computer/ over the older website documentation index.

## References used for the notes

| Area | Primary sources |
| --- | --- |
| Introduction | [Website](https://www.iroh.computer/), [What is Iroh](https://docs.iroh.computer/what-is-iroh), [quickstart](https://docs.iroh.computer/quickstart) |
| Connection model | [Endpoints](https://docs.iroh.computer/concepts/endpoints), [address lookup](https://docs.iroh.computer/concepts/address-lookup), [NAT traversal](https://docs.iroh.computer/concepts/nat-traversal), [relays](https://docs.iroh.computer/concepts/relays) |
| Sharing and access | [Tickets](https://docs.iroh.computer/concepts/tickets), [security/privacy](https://docs.iroh.computer/concepts/security-privacy), [hooks](https://docs.iroh.computer/connecting/endpoint-hooks) |
| Protocols | [Overview](https://docs.iroh.computer/concepts/protocols), [blobs](https://docs.iroh.computer/protocols/blobs), [gossip](https://docs.iroh.computer/connecting/gossip), [documents](https://docs.iroh.computer/protocols/documents), [Automerge](https://docs.iroh.computer/protocols/automerge), [streaming](https://docs.iroh.computer/protocols/streaming) |
| Integration | [Rust](https://docs.iroh.computer/languages/rust), [Node.js](https://docs.iroh.computer/languages/javascript), [Python](https://docs.iroh.computer/languages/python), [browser/Wasm](https://docs.iroh.computer/languages/wasm-browser), [compatibility](https://docs.iroh.computer/compatibility) |
| Operations | [Enterprise networks](https://docs.iroh.computer/configuring-networks), [public relays](https://docs.iroh.computer/iroh-services/relays/public), [self-hosted relays](https://docs.iroh.computer/iroh-services/relays/self-hosted), [release policy](https://docs.iroh.computer/about/release-policy), [FAQ](https://docs.iroh.computer/about/faq) |
| Version-pinned API | [Endpoint source at v1.3.0](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/src/endpoint.rs), [presets](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/src/endpoint/presets.rs), [echo](https://github.com/n0-computer/iroh/blob/v1.3.0/iroh/examples/echo.rs) |
| Runnable references inspected | [Ping quickstart source](https://github.com/n0-computer/iroh-ping/blob/main/examples/quickstart.rs), [sendme README](https://github.com/n0-computer/sendme/blob/main/README.md) |
| Release evidence | [GitHub v1.3.0](https://github.com/n0-computer/iroh/releases/tag/v1.3.0), [crates.io API](https://crates.io/api/v1/crates/iroh), [npm API](https://registry.npmjs.org/@number0%2firoh/latest) |

The docs pages expose Markdown by appending `.md`, useful for future research.
Start with [the current docs index](https://docs.iroh.computer/llms.txt).

## Evidence files and limits

- [sources.json](sources.json) records retrieval URLs, access date, content sizes,
  SHA-256 hashes, and whether a resource was inspected or only collected for reference.
- [version-baseline.json](version-baseline.json) preserves the small subset of
  release/registry metadata used to establish the baseline.
- The notes are our synthesis, with source links near the relevant claims.
  They are not a vendored copy of the upstream documentation. Content hashes
  identify what was retrieved; hashes alone cannot reconstruct those pages.
- Links to example directories in prototype-options.md are suggested next reads
  unless explicitly marked as inspected. The main-branch links can change.
- No code was compiled or run. No measured performance or connectivity results
  should be inferred from this research.
- Pricing was linked for future evaluation; a pricing plan and detailed price
  schedule were not researched or selected.
