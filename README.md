# Iroh lab · Pocket link

A static phone PWA that connects to a native sandbox peer through
[Iroh](https://www.iroh.computer/). Pair with a private invitation, exchange text
messages, and measure round-trip time. Browser connections are end-to-end
encrypted and relayed; the UI reads the actual selected Iroh path.

- **Phone app:** https://d3mnjx.github.io/iroh-lab/
- **Interactive diagram:** https://d3mnjx.github.io/iroh-lab/diagram.html
- **Repository:** https://github.com/d3mnjx/iroh-lab
- **Full implementation plan:** [PLAN.md](PLAN.md)
- **Validation and limits:** [TESTING.md](TESTING.md)
- **Original research:** [overview](research/overview.md),
  [implementation notes](research/implementation-notes.md),
  [prototype options](research/prototype-options.md), [sources](research/sources.md)

## Try it on your phone

1. Start the sandbox peer using the commands below.
2. Open the private link in `runtime/pairing-url.txt` on your phone. You can also
   open the phone app and paste that link into **Private invitation**.
3. Tap **Connect to sandbox**. Send a message or tap **Ping**.
4. From a sandbox terminal, send a reply:

   ```bash
   ./target/debug/iroh-lab-peer send 'Hello from the sandbox'
   ```

For a QR code, run `npm run qr` and view `runtime/pairing-qr.png` locally.
Treat that QR and invitation link as private: anyone holding them can join the
current session. Never upload them to the repository, Pages, or issue reports.

The PWA can be installed through the browser menu. On iPhone, use Safari's
**Share → Add to Home Screen**. The installed app has the same relay and mobile
background limitations as the browser app.

## Build and run the sandbox peer

Prerequisites: Rust/rustup, a C compiler and linker, Node.js 22, and npm.
The pinned Rust toolchain is installed automatically through rustup when needed.
On Debian/Ubuntu, native and Wasm builds use `build-essential pkg-config
libssl-dev clang`. The browser binding generator is pinned to 0.2.122.

```bash
cargo build --locked --features native
npm ci
npm run peer:start
```

`peer:start` launches the native peer detached, waits for readiness, and saves
its PID and log in `runtime/`. It reuses a healthy existing peer. For foreground
operation instead:

```bash
./target/debug/iroh-lab-peer serve
```

The first start creates a persistent endpoint key. Each new server process creates
a fresh invitation token, invalidating previous invitations. This preserves the
endpoint ID while rotating session access. Use a separate `--state-dir` for each
independent peer; the process locks its directory to prevent duplicate identities.

```bash
./target/debug/iroh-lab-peer status
./target/debug/iroh-lab-peer send 'A message for the phone'
```

`status` shows the first batch (up to 40) of retained messages. There is a bounded
in-memory history of 200 messages shared by devices using this session's invitation.
Nothing is a durable inbox. Messages disappear when the server restarts.

To stop a detached peer, send SIGINT to the process listed in `runtime/peer.pid`
after checking it is still this project's peer. Restart with `npm run peer:start`
and obtain the new invitation. Sandbox lifetime determines availability.

## Build and serve the phone app locally

```bash
bash scripts/install-wasm-bindgen.sh  # Prebuilt generator for Linux x86_64
npm run build
npm run serve
```

On other platforms install the same generator version with
`cargo install wasm-bindgen-cli --version 0.2.122 --locked`.
Open http://127.0.0.1:4173/iroh-lab/ and paste the invitation. Only `dist/` is
published. No Rust executable or runtime credentials are part of the site.

The workspace's host-mounted filesystem does not support npm binary symlinks.
In that environment use `npm ci --no-bin-links`; the npm scripts invoke tools by
explicit paths. Set `CARGO_INCREMENTAL=0` to avoid host-mount incremental-cache
permission warnings. Ordinary Linux CI does not need these workarounds.

## Checks

```bash
cargo fmt --check
cargo test --locked --features native
cargo clippy --locked --features native --all-targets -- -D warnings
npm test
npm run build
node node_modules/@playwright/test/cli.js install chromium
# With the peer and local static server running:
npm run test:browser
```

For Safari-engine connection checks, install WebKit and its system dependencies,
then run:

```bash
node node_modules/@playwright/test/cli.js install --with-deps webkit
PLAYWRIGHT_BROWSER=webkit npm run test:browser -- --grep 'real browser|wrong invitation'
```

This uses desktop Linux WebKit with an iPhone viewport, not an actual iPhone.
The separate offline-reload test currently encounters a Linux WebKit automation
error; see TESTING.md. Chromium's offline-shell test passes.

For deployed testing set `PWA_URL=https://d3mnjx.github.io/iroh-lab/`.
`IROH_TEST_RUNTIME`, `IROH_PEER_BINARY`, and `PLAYWRIGHT_CHROMIUM_EXECUTABLE` can
select another local test peer/binary/browser. Real network tests depend on
public relay availability. Test artifacts are ignored; traces and screenshots
are disabled by default because pairing URLs are credentials.

## Deployment

The Pages workflow tests the protocol, builds Rust/WebAssembly and the static
assets, and deploys through GitHub Actions. Pages must use **GitHub Actions** as
its source. Push to `main` or dispatch the workflow to deploy.

The manifest, asset URLs, and service-worker scope support the `/iroh-lab/`
project path. A content-derived cache version keeps a coherent application shell
and Wasm build. Updates wait until existing app tabs are closed. No invitation,
chat history, or third-party relay traffic is stored in the service-worker cache.

## Boundaries

- GitHub Pages hosts the app files. Iroh relays carry encrypted peer traffic.
- The browser uses the relay explicitly carried by the invitation. Its hostname
  is normalized for Safari TLS validation, and the browser does not enable public
  address lookup. If the sandbox moves to another relay, obtain a fresh invitation.
- Both peers must be online. A cached offline app shell cannot send messages.
- Mobile browsers may suspend an open connection; reconnect after reopening.
- Invitations grant session access and are kept in memory. Reloading clears them;
  keep your original private link if you want to connect again.
- Message contents stay in the endpoints; relays still see connection metadata.
- No shell execution, file access, accounts, push notifications, or paid services.
- This is a prototype with public development relays and no availability SLA.

## Layout

```text
src/lib.rs            Shared bounded protocol and validation
src/browser.rs        Browser Iroh/Wasm adapter
src/bin/peer.rs        Native peer and loopback-only operator control
web/                  Phone interface, manifest, icons
scripts/              Build, service worker, launcher, QR generation
research/             Dated notes and primary sources
runtime/              Private local keys/invitations/logs (ignored)
dist/                 Generated static site (ignored)
```
