# Plan: phone PWA connected to a sandbox peer over Iroh

Accepted direction: 2026-10-16. This implements the user's request to build the
previously discussed sandbox-to-phone experiment, create its GitHub repository,
and publish the phone app on GitHub Pages.

## Intended result

Open https://d3mnjx.github.io/iroh-lab/ on a phone, install it as a PWA if desired,
open a private pairing link (or paste it), and exchange messages with a running
peer in this sandbox. Observe connection state and ping round-trip time. The
phone's Iroh transport is WebAssembly and uses an encrypted relay path.

GitHub owner: d3mnjx, the authenticated account available in this workspace.
Repository: public d3mnjx/iroh-lab, with origin pointing to that repository.
The user explicitly authorized repository creation and Pages deployment.

## Architecture

- Static HTML/CSS/JavaScript PWA on GitHub Pages, with a Rust/WebAssembly adapter
  for Iroh 1.3.0. No cloud application backend, analytics, or external fonts.
- Native sandbox peer sharing the Rust protocol where practical. It keeps a
  persistent local endpoint identity and serves a small, bounded messaging API.
- Public Iroh relays for the experiment. No paid service or API key required.
- An application-specific, versioned ALPN; bounded JSON messages over QUIC.
- A high-entropy invitation token authorizes the session. Pairing details live
  in the URL fragment, which is not sent in HTTP requests to GitHub Pages.
  The invitation includes the endpoint address/relay so pairing does not depend
  solely on public address lookup propagation. Strip the fragment after import.
- The native peer prints/writes its invitation locally. No private keys,
  invitation tokens, chat logs, or runtime state are committed or built into Pages.
- The sandbox peer supports operator-originated messages through a local CLI
  control path. The phone can send messages, receive messages, and measure ping.
  The scope is messaging; arbitrary commands and filesystem access are excluded.

## Phone experience

1. Load the app; explain briefly that the sandbox peer must be running.
2. Open/paste an invitation. Show the peer fingerprint and connect action.
3. Display clear connecting/connected/disconnected/error states, with bounded
   attempts and a reconnect option after mobile suspension or network loss.
4. Exchange text messages with the sandbox. Render text as text, never HTML.
5. Offer a ping action and show measured round-trip time. Describe the browser
   path as relayed, without implying a direct phone-to-sandbox connection.
6. Provide disconnect/forget controls. Keep the token in session memory by default.
7. Support small phone screens, keyboard use, readable contrast, safe-area insets,
   and reduced motion. Do not put build/API details in the ordinary user flow.

## PWA and hosting

- Manifest with app name, icons, standalone display, project-relative scope and
  start URL. GitHub Pages project path must work without root-relative assets.
- Service worker caches the versioned application shell and Wasm for reopening
  after an initial load. Offline launch is distinct from live peer connectivity.
- Avoid caching invitations, chat data, or arbitrary network traffic. Use an
  update strategy that prevents JavaScript/Wasm version mismatches.
- Repository Actions builds pinned Rust/Wasm dependencies, runs appropriate
  checks, uploads the static artifact, and deploys with official Pages actions.
- Enable Pages using GitHub Actions and verify the actual public HTTPS URL.

## Implementation sequence

1. Save this plan before implementation; retain the earlier research.
2. Verify pinned APIs and set up toolchains/build dependencies.
3. Implement and test the bounded shared protocol and native peer, including
   persistent identity, invitation authorization, and clean shutdown.
4. Implement the Wasm adapter with deadlines, bounded messages, reconnect and
   disconnect behavior. Build against the same protocol as the native peer.
5. Implement the responsive phone UI, manifest, icons, and service worker.
6. Run a real browser-to-native test through an Iroh relay: authorize, send,
   receive an operator message, ping, reject an invalid invitation, reconnect.
7. Check PWA assets, offline shell, mobile viewport, and absence of leaked
   runtime credentials in tracked/published files.
8. Create the public repository, push reviewed files and lockfiles, enable
   Pages, run deployment, and verify the hosted build against the sandbox peer.
9. Record test evidence, launch/restart instructions, known limitations, and
   final URLs. Keep the sandbox peer running for the user's first phone test
   where this environment permits it.

## Success criteria

- Static site and Wasm load from the real GitHub Pages project URL over HTTPS.
- Manifest and service worker use the correct project scope; app shell reopens
  offline after initial caching. Installation still depends on phone/browser UI.
- A browser using the published build connects to the sandbox through a public
  relay and exchanges text in both directions. Ping shows a measured value.
- Invalid/missing authorization is rejected before protected messaging.
- Oversized/invalid requests are bounded; network failure produces a recoverable
  state rather than an indefinite spinner or silently lost confirmed message.
- Restarting the sandbox keeps its endpoint ID; rotating the invitation token
  revokes the previous invitation for new sessions.
- Secrets and runtime data remain local and ignored by Git.

## Testing and evidence boundaries

Unit tests focus on framing limits, parsing, and authorization where practical.
Browser integration tests exercise the real Wasm/native path, not a mock transport.
Record browser version, core version, path observations, and exact outcomes in
TESTING.md. Do not claim a physical-phone test until the user runs one. A browser
in this environment proves the integration, but not a particular cellular carrier
or the phone's installation/background behavior.

## Constraints and limitations

- Browser Iroh currently requires relays. Public relays have rate limits and no SLA.
- Both peers must be online. Messages are ephemeral; there is no offline inbox.
- Mobile browsers may suspend the app. Reconnect is expected when reopening it.
- A running sandbox process is temporary infrastructure; its availability depends
  on this environment staying alive. Document the restart command.
- Iroh protects transport contents, not all connection metadata. The invitation
  is a bearer credential: anyone possessing it can join until rotated/restarted
  according to the implemented lifecycle.
- This is an exploration prototype, not a production multi-user messaging system.

## Deferred work

File transfer, gossip rooms, durable history, push notifications, background
execution, user accounts, custom relay hosting, and richer document sync remain
future choices after testing this small end-to-end system.
