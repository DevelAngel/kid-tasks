---
status: accepted
date: 2026-07-10
---

# rmcp for MCP-Based Agent Communication

## Context and Problem Statement

AI assistants currently talk to `kid-server` through the `kid` CLI, which
itself is a tarpc client over TCP (see [tarpc for Rust-Native RPC Communication](tarpc-for-rust-native-rpc-communication.md)). Every
operation costs a process spawn, a TCP round-trip, and a hand-rolled
JSON-quoting exercise on the shell command line. Modern AI assistants
(Claude Desktop, Claude Code, Goose, …) speak the [Model Context Protocol](https://modelcontextprotocol.io/) natively — tool discovery,
typed JSON arguments, and structured results are handled by the client
runtime instead of a bespoke CLI wrapper. How should `kid-server` expose
its task operations to AI assistants going forward?

## Decision Drivers

- Native tool-calling support in AI assistant runtimes — no CLI parsing detour
- JSON arguments passed as real objects, not shell-quoted strings
- Read-only reference data (categories, contexts, domain guide) should be
  discoverable without invoking a "verb"
- Assistant identity must not be spoofable by a freely-chosen CLI flag
- Keep the browser-facing HTTP server free of agent traffic and unrelated
  attack surface (Tinyauth forward-auth protects the browser path only)
- Tool descriptions are read into the model's context on every session —
  the entire tool set must fit comfortably inside a small local model's
  context window (~4K tokens for the Jetson-based local LLM, see `docs/ai/local-llm-server.md`)
- Minimize scope for the initial cut (MVP) — defer anything not required
  for core task mutation/reading

## Considered Options

- rmcp (official Rust MCP SDK) as a standalone TCP/HTTP server, separate port
- rmcp mounted into the existing Axum HTTP server (same port as the browser UI)
- Keep tarpc, add an MCP-to-tarpc translation shim
- JSON-RPC over HTTP (hand-rolled, no MCP framing)

## Decision Outcome

Chosen option: "rmcp as a standalone Streamable-HTTP server on its own
port", because it gives AI assistants a native, typed tool-calling
interface while keeping agent traffic fully separate from the
browser-facing, Tinyauth-protected HTTP server. `kid-cli` is deleted
outright — no CLI, no tarpc, no shell-quoting layer remains.

### Transport and port

`kid-server` gains a third listener (alongside the existing HTTP/SSE
port and the Leptos asset server) dedicated to MCP: `rmcp`'s
Streamable-HTTP transport (`transport-streamable-http-server`,
stateful mode, `LocalSessionManager`) served directly via `axum::serve`
on its own `SocketAddr`, configured the same way the old `--listen`
RPC address was (default `127.0.0.1:9100`, override via CLI flag / env).

This was chosen over mounting `/mcp` into the existing browser router
because Tinyauth (see [Tinyauth for Public Internet
Authentication](tinyauth-deployment.md)) fronts the browser port with
forward-auth login flows that AI assistant HTTP clients cannot complete
(no cookie jar, no interactive login). A separate, unauthenticated
(trusted-network-only) port for MCP avoids forcing agents through a
human login flow, mirroring how the old RPC port was never behind
Tinyauth either.

### Assistant identity

The old CLI required an explicit `--assistant` flag (default
`"assistant"`, override via `KID_CLI_ASSISTANT`) that any caller could
set to anything, including impersonating a different assistant. MCP's
`initialize` handshake carries `clientInfo.name`, but that's supplied by
the client's own runtime and thus just as freely claimed — no better
than the old flag.

Instead, both halves of the actor string (`<name>:<on_behalf_of>`,
see `docs/concepts/author-tracking.md`) are fixed server-side, per
OAuth client, in `--mcp-clients-file`: `name` (optionally prefixed,
e.g. `ai:claude-desktop` vs. `matrix-relay` — not every client is an
AI assistant) is configured separately from the `client-id` the client
authenticates with, and `on_behalf_of` names the human that client
acts for. `validate_access_token`
stashes both in the request's extensions after authenticating the
client, and `McpService::actor` reads them back from there; neither is
a tool parameter, so no client can claim to be someone else or act on
someone else's behalf.

Fallback when a piece is unexpectedly missing from the request
extensions (should not happen for a request that made it past
authentication): that piece falls back to `"unknown"`.

### Tools vs. Resources

This is the project's first use of MCP resources. The split:

Tools carry the mutations, since every one of them needs typed
parameters: `add`, `rename`, `replace`, `update`, `complete`,
`recategorize`, `add_contexts`, `replace_contexts`, `set_priority`.
`list` (with an optional search/status filter) is a tool too, for the
same reason — it takes parameters, so it is not a fixed document.

Resources cover static reference data that never takes parameters:
`kid://categories` and `kid://contexts` (snapshots of categories and
contexts currently in use), and `kid://guide` — the domain guide
(status shape, priority letters, category/context conventions)
migrated from `docs/ai/SKILL.md`, fetched once and cached by the
client instead of re-sent in every prompt.

A `kid://schema/task-details` resource (JSON Schema for task details,
equivalent to the old `kid schema` command) was considered and
rejected: `details` is now passed as a native JSON object directly in
tool call arguments (see below), and the tool's own JSON-Schema input
definition — which every MCP client already fetches and shows the
model — serves the same documentation purpose. A separate schema
resource would duplicate that information for no benefit.

### JSON details as native objects

`add`, `replace`, and `update` accepted a JSON _string_ in the CLI
(`--details '{"priority":"A"}'`), a documented pain point in the
superseded [CLI API Design](cli-api-design.md) ADR ("JSON strings
require shell quoting"). MCP tool arguments are structured JSON
natively, so `details` becomes a regular nested object field in the
tool's input schema — no quoting, no string-escaping round-trip, no
`serde_json::from_str` step in the handler.

### Tool description budget

Tool and parameter descriptions are loaded into context on every
session and must stay small enough for the local 3B-class model
mentioned in `docs/ai/local-llm-server.md` (4K context window) to
still have room for the conversation itself. Concretely: the combined
size of all tool descriptions, parameter descriptions, and the
`kid://guide` resource must stay well under 4K tokens. This rules out
verbose per-field prose (e.g. spelling out every enum value of
`TaskCategory`/`TaskContext` in a description) — such reference data
belongs in the `kid://guide` resource, fetched once, not repeated in
every tool schema.

### MVP scope — server-control operations kept off the MCP server

`switch_dir`, `set_time_offset`, `reset_time_offset`, `count`, and
`flush` existed purely to support the e2e browser test harness, not
actual task management. Exposing them as regular MCP tools on an
otherwise trusted-network port would be an unresolved safety question
(`switch_dir` accepts an arbitrary filesystem path; `set_time_offset`
warps time for every connected client) — so the rmcp server ships
**task tools and resources only**, and stays that way.

The e2e harness instead gets its own small JSON-over-HTTP admin
channel (`kid-server/src/testctl.rs`), compiled in only behind the
`test-control` Cargo feature (`cargo leptos end-to-end --bin-features
kid-server/test-control`) and bound only when
`KID_TEST_CONTROL_ADDR`/`--test-control-listen` is explicitly set.
Production builds never enable the feature, so the admin channel does
not exist in the shipped binary at all — two independent gates
(compile-time feature, explicit address) instead of relying on either
alone. Real task mutations the harness needs (e.g. `add` for the
flush-LED test) go through the ordinary MCP client instead.

### Consequences

- Good, because AI assistants get native, discoverable tool calling —
  no CLI process spawn, no shell escaping
- Good, because assistant identity is no longer a freely-chosen string
  parameter — it comes from the MCP handshake
- Good, because `details` payloads are structured JSON end-to-end,
  removing an entire class of quoting bugs
- Good, because resources are used for genuinely static reference data,
  keeping tool schemas small
- Good, because agent traffic and browser traffic stay on separate
  ports — Tinyauth's forward-auth flow never has to deal with
  non-interactive MCP clients
- Good, because the CLI-driven `docs/ai/SKILL.md` becomes obsolete —
  its content lives as an MCP resource instead, fetched directly by
  the assistant rather than maintained as a separate prompt file
- Bad, because a second unauthenticated port is additional attack
  surface if `kid-server` is ever exposed beyond a trusted network —
  mitigated by binding to a private/VPN-only interface, same as the
  old RPC port always was

## Pros and Cons of the Options

### rmcp as a standalone server, separate port (chosen)

- Good, because clean separation from Tinyauth-protected browser traffic
- Good, because no interactive-login problem for non-browser clients
- Bad, because one more port to firewall/document

### rmcp mounted into the existing Axum HTTP server

- Good, because a single listener, simpler deployment story
- Bad, because Tinyauth's forward-auth middleware sits in front of that
  listener and expects an interactive login flow AI assistants cannot
  perform
- Bad, because carving out an unauthenticated exception path (`/mcp`)
  inside an otherwise authenticated router is a more fragile security
  boundary than a physically separate port

### Keep tarpc, add an MCP-to-tarpc translation shim

- Good, because no change to the existing `TaskService` trait
- Bad, because adds a translation layer instead of removing one —
  exactly the complexity this change is meant to eliminate
- Bad, because AI assistant runtimes still need something to translate
  _to_ MCP in the first place, so the shim buys nothing over
  implementing rmcp directly

### JSON-RPC over HTTP (hand-rolled)

- Good, because simple, no new dependency
- Bad, because reinvents tool discovery, schema advertisement, and
  session handling that MCP already standardizes and that AI assistant
  runtimes already implement clients for

## More Information

**Crate:** `rmcp` (workspace-pinned at `2.2.0`), features `server`,
`server-side-http`/`transport-streamable-http-server`, `schemars`.
`kid-types` keeps its `schemars`-enabling feature (used for tool input
structs) but drops `tarpc` and `clap` entirely — the `rpc` and `cli`
features and `src/rpc.rs` are removed.

**Removed:** `kid-cli` crate in full (binary, `TaskServiceClient`
re-export, `connect()` helper). `kid-server/src/rpc.rs` is replaced by
an expanded `kid-server/src/mcp.rs`; `ServerBuilder` drops its
`rpc_addr` builder step in favor of an `mcp_addr` step spawning the new
listener alongside the HTTP listener.

**Follow-up work (not part of this ADR, see Open above):**

- `docs/ai/SKILL.md` → content migrates into the `kid://guide` MCP resource
- `docs/ai/local-llm-server.md` → CLI-driving language needs updating to MCP tool-calling language
- `docs/test-instructions.md` → RPC references become MCP references
