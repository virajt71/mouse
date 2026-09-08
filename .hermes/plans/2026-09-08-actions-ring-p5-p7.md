# Actions Ring — Execution Board

## DONE (P1-P4 + this session)
- [x] P1 data model + proto
- [x] P2 engine hook + gRPC
- [x] P3 ring overlay module
- [x] P4 config UI + sidebar
- [x] trigger path: actions_ring_open -> trigger_ring_open -> counter
- [x] GUI poll get_ring_trigger_count -> open ring at cursor
- [x] update.rs per-frame handle+draw
- [x] Config::active_profile_ring_layout()
- [x] BUGFIX: instant-close (last_active_time=0.0 sentinel)
- [x] P7 fade-out on close (begin_close + closing flag)
- [x] BUGFIX: ring trigger never broadcast -> GUI never saw it
      (execute_engine_action RPC now pushes config_to_proto snapshot via config_tx)
- [x] per-app ring layout auto-switch: handle_app_change now persists
      active_app_profile and broadcasts config_generation so GUI reads
      per-app ring layouts on app switch

## VERIFIED
- [x] engine/tests/ring.rs — 28 tests covering state machine, geometry,
      hit-test, clamp, layout resolution, constants (all pass)

## DEFERRED (non-blocking)
- [ ] toast on action error: execute_engine_action returns (), nothing to surface.
      (GUI would need a separate error-channel / return-value from execute_engine_action.)
