# P0 — Immediate / Safety Issues (Auto-generated)

Date: 2025-12-08

This file lists the top-priority, immediate safety issues discovered by an automated scan. Each entry includes a suggested fix and why it's high-risk.

1) `src/network/server.rs` — unsafe locking / unwrap in runtime accept loop
- locations (examples):
  - `*self.running.lock().unwrap() = true;` (start)
  - `let mut clients_guard = clients_clone.lock().unwrap();` (accept loop)
  - `if !*running_clone.lock().unwrap()` / `while *running_clone.lock().unwrap()` (shutdown loops)
- Why P0: Unhandled `PoisonError` or other lock poisoning will panic the server process; this code runs in production path and handling must be made fallible or resilient.
- Suggested fix: Replace `.lock().unwrap()` with robust handling using `match` on `.lock()`, use `into_inner()` on PoisonError when safe, or propagate an error (change API to return Result). Add tests and a short rollout plan.

2) `src/network/mod.rs` — `delta_serializer.as_ref().unwrap()` and subsequent locked serializer
- locations: `state.delta_serializer.as_ref().unwrap();` (uses `unwrap()` in runtime code)
- Why P0: unwrap here might panic at runtime (missing serializer state). Replace with Option handling or ensure serializer is always initialized and check at startup.

3) `src/network/compression.rs` — multiple `.unwrap()` uses in compress/decompress runtime paths
- Why P0: Compression failures should be handled gracefully — unwrapping may abort connection handling.

4) `src/performance/memory/arena.rs` — `unsafe` usage with `alloc`/`dealloc` and `Layout::from_size_align(...).unwrap()`
- Why P0: incorrect layout assumptions or `unsafe` misuse can cause memory UB and crash or corruption.
- Suggested fix: Add validation, documented invariants, property tests, and run MIRI for these modules.

5) Misc network parse / get_session unwraps in server
- `let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();` — mostly in tests/examples. If this pattern exists in runtime configuration parsing, fail gracefully instead.

Recommended next actions (short):
- Create issues for each entry above linking to file+line and suggested remediation; mark them P0 and add an owner.  
- Apply quick fixes (replace critical `.lock().unwrap()` usage in `src/network/server.rs` with poison-safe locking or Result propagation).  
- Add CI gating to prevent new critical unwraps in non-test code.
