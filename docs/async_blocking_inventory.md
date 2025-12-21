# Async Boundary Inventory

Scanned for `block_on`, `Handle::current().block_on` and `pollster::block_on` occurrences and classification.

## Findings (auto-collected)

- `game_engine/tests/async_io_tests.rs:178` — `tokio::runtime::Handle::current().block_on(...)` — tests: keep as test helper or refactor to `#[tokio::test]`.
- `game_engine/examples/xr_demo.rs:292` — `pollster::block_on(instance.request_adapter(...))` — examples: acceptable but mark as "demo-only" or gate behind feature.
- `game_engine/examples/xr_demo.rs:298` — `pollster::block_on(adapter.request_device(...))` — examples: same as above.
- `game_engine/benches/render_benchmarks.rs:121,138,260,277` — uses `.block_on()` in benches: change to smaller synchronous mocks for unit tests and use `tokio::test` or `pollster` in benches only.
- `game_engine/src/core/scheduler.rs:410` — `scheduler.block_on(...)` (custom API) — evaluate if this wrapper uses runtime appropriately; may be acceptable.
- `game_engine/src/core/engine/renderer.rs:371,400` — `tokio::runtime::Handle::current().block_on(...)` used during rendering initialization & file IO — needs async conversion or `spawn_blocking`/offload.
- `game_engine/src/plugins/hot_reload.rs:67,120,226,254` — hot-reload uses `block_on` frequently — prefer async or `spawn_blocking` to avoid blocking runtime.
- `game_engine/builtin/render.rs` — commented `pollster::block_on` call — keep commented or replace.
- `game_engine/src/resources/runtime.rs:35,58` — provides `block_on` convenience function — should be reviewed for use sites and limited to startup code or testing.
- `game_engine/src/resources/manager.rs:244` — `rt.block_on` usage — review.
- `game_engine/src/platform/mod.rs:333-340` — platform filesystem uses `tokio::runtime::Handle::current().block_on(self.read(path))` — should be made async or use `block_in_place` in non-runtime threads and return errors when called from runtime.
- `game_engine/src/network/client.rs:557` and `server.rs:992` — network client/server using `block_on` — need audit.
- `game_engine/src/profiling/storage.rs` and `game_engine_performance/src/profiling/storage.rs` — multiple `Handle::current().block_on` — should be refactored to async or async-friendly API with sync wrappers guarded.
- `game_engine/src/scene/serialization.rs:562,569` — block_on used for save/load wrappers — convert to async save/load or provide sync methods that detect runtime.
- `game_engine/src/editor/project_settings.rs:196,203` — editor save/load uses `block_on` — handle via editor main thread (which may be sync) or provide async editor API.

## Next actions

1. Classify each occurrence as:
   - `Whitelist` (acceptable: tests, examples, benches, startup-only code)
   - `Refactor` (must become async or use `spawn_blocking`/offload)
   - `Protect` (wrap with runtime-detection and return error or use `block_in_place`)

2. Create a linting script: `scripts/find_blocking_calls.sh` which fails CI if new unapproved `block_on` calls are added.

3. Start triage in priority order:
   - Fix platform filesystem (high-priority)
   - Editor save/load (high-priority)
   - Profiling/storage (medium-priority)
   - Plugins/hot-reload (medium)
   - Replace global `block_on` helpers or document their allowed context (startup/testing only)

---

I'll start triaging files by adding TODO comments and opening small, focused patches for P0-3 fixes.