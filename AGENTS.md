# Iroh lab context

Read README.md and research/implementation-notes.md before implementation.

- The user approved the sandbox-to-phone PWA prototype, public repository
  creation under the authenticated d3mnjx account, and GitHub Pages deployment.
  PLAN.md is the accepted implementation direction; other research prototype
  options remain proposals.
- Keep project-specific research, code, and test evidence in this repository.
- Date research and cite primary sources. Distinguish documented behavior,
  inspected source, measured results, and assumptions.
- Use version-matched source/API references. The documentation contains mixed
  API generations; research/implementation-notes.md records concrete examples.
- When implementing, pin a compatible dependency set and preserve its lockfile.
- Never describe a successful connection as direct without inspecting its path.
  Browser Iroh currently uses relays; a same-machine test cannot validate WAN NAT traversal.
- Keep private keys, capability tickets, credentials, and runtime data out of Git.
- Use Git identity d3mnjx <339244255+d3mnjx@users.noreply.github.com>.
