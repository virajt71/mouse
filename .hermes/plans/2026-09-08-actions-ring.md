# Actions Ring — Feature Plan

## Overview

Build the "Actions Ring" feature: a radial menu overlay triggered from a mouse button, displaying app-aware actions as bubbles arranged in a circle, with hover/click activation firing configured key-combo actions back to the engine. Mimics Logi Options+ Actions Ring (v1.83+).

## Architecture

Two crates in a workspace:
- **engine** (`mouser_engine`): daemon doing HID++ I/O, action dispatch, config persistence, gRPC server
- **gui** (`mouser_gui`): egui 0.29 desktop app, talks to engine via gRPC over Unix socket

IPC contract lives in `engine/proto/mouser.proto`. When `.proto` changes, run `cargo clean -p mouser_engine` to force proto regeneration (build.rs timestamp detection is unreliable).

## Milestones

### P1 — Data model & proto (engine)

Add `ring_trigger_count: u64` to `ConfigResponse`, add `ExecuteActionRequest/Response` messages and `ExecuteEngineAction` RPC to `engine/proto/mouser.proto`.

Add to `engine/src/config.rs`:
- `RingLayout`, `RingBubble`, `RingBubbleKind`, `RingFolder`, `RingAdjustmentRange` structs (derives: Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)
- `ring_layout: Option<RingLayout>` field on `Profile`
- `ring_trigger_count: u64` on `EngineState`

**Done**: proto updated, config structs added.

### P2 — Engine hook & gRPC (engine)

- Add `increment_ring_trigger_count()` and `trigger_ring_open()` methods to `Engine`
- Add `ring_trigger_count: u64` to `EngineState` in `engine/src/engine/inner.rs`
- Wire `actions_ring_open` handler in `execute_engine_action` dispatch to call `self.trigger_ring_open()`
- Add `ring_layout: None` to manual `Profile` constructors in `engine/src/engine/profile.rs`
- Add `ring_trigger_count` to `broadcast_config` in `engine/src/grpc/server.rs`
- Implement `ExecuteEngineAction` RPC handler in `engine/src/grpc/server.rs`
- Update `last_config` tuple in `engine/src/client.rs` to include `ring_trigger_count`; implement `execute_action` client method

**Done**: all wired.

### P3 — Ring overlay GUI (gui)

New module `gui/src/app/ring.rs`:
- `RingState` struct, `new()`, `update()`, `draw_ring()`, `handle_ring_input()`, `draw_adjustment_bar()`
- egui 0.29 compatible API — no `InteractionOutput`, proper `Response` handling
- Ring overlay renders bubbles in a circle centered on cursor position

Wire into `gui/src/app/mod.rs`:
- `mod ring;` declaration
- `ring_overlay: Option<RingOverlay>` field on `MouserApp`
- `ring_overlay.update()` in update loop
- `ring_overlay.draw_ring()` in render loop

**Done**: ring module created and integrated.

### P4 — Config UI & sidebar (gui)

- `gui/src/views/customization/tabs/ring_tab.rs` — ring configuration tab
- `gui/src/views/customization/popups/ring_action_picker.rs` — action picker popup
- `gui/src/views/customization/popups/ring_folder_popup.rs` — folder editor popup
- `gui/src/views/customization/sidebar.rs` — add `Ring` variant to both keyboard and mouse match arms
- Add `actions_ring_open` to default button mappings for `Button::Thumb` in `engine/src/config.rs`

**Done**: all created and wired.

### P5 — Per-app ring layout auto-switch (engine)

`handle_app_change` needs to persist `active_app_profile` and broadcast `config_generation` so the GUI reads per-app ring layouts on app switch. Currently `handle_app_change` updates `current_profile` but not `config.active_app_profile`.

**Done**: `app_change.rs` patched to persist and broadcast on app switch.

### P6 — Ring test coverage (engine + gui)

Add `engine/tests/ring.rs` covering:
- `RingState` default/reset/open/begin_close/fade_out lifecycle
- `compute_bubble_positions` geometry (count, equispacing, first-bubble-up, arbitrary radius)
- `hit_test_bubbles` (hits, misses, empty list, correct bubble, border case, disambiguation)
- `clamp_ring_to_screen` (clamps to bounds, allows full when fits, tight single-point screen)
- `is_numeric_action_id` (recognizes volume/zoom, rejects others)
- `get_primary_bubbles` / `get_folder_bubbles` / `get_ring_layout` resolution
- Constants relationship

Add `mouser_gui = { path = "../gui" }` to `engine/Cargo.toml` `[dev-dependencies]`.
Add `egui` to the same `[dev-dependencies]` block so tests can use egui types directly.

**Done**: 28 tests written, all passing.

### P7 — Polish (gui)

- **Toast on action error**: `execute_engine_action` currently returns `()`, so there's nothing to surface. Would need a separate error channel or return-value from `execute_engine_action`. Deferred.
- Ring icon rendering via texture atlas instead of text labels (P4 ring_tab already uses icon_name field; swap when atlas lands).
- Per-app ring layout auto-switch verification — manual QA.

## Dependency chain

```
P1 (proto+config structs) ──┐
                             ├──▶ P2 (engine methods + gRPC) ──┐
P3 (ring overlay GUI) ───────┘                                ├──▶ P4 (config UI + sidebar)
                                                              └──▶ P5 (per-app switch)
                                                                      └──▶ P6 (tests)
                                                                          └──▶ P7 (polish)
```

P1 and P3 are parallel-safe (engine data model + GUI overlay don't touch each other). P2 depends on P1 (needs the proto messages + config structs). P4 depends on P2 (needs ExecuteEngineAction RPC + ring_trigger_count in config stream). P5 depends on P2 (needs broadcast_config working). P6 can run in parallel after P3/P4 land. P7 is last.

## Constraint notes

- Rust edition 2021, egui 0.29 API, Tonic gRPC over Unix socket
- Proto regeneration must be triggered manually (`cargo clean -p mouser_engine`) after `.proto` changes
- No new external dependencies without justification — egui+`eframe`+`tray-icon` already in gui/Cargo.toml
- Avoid global mutable state where possible; prefer passed-in state or Arc<Mutex<>>

## Test strategy

- Every new public function gets a test in `engine/tests/` or a `#[cfg(test)] mod tests` in its source file
- Integration tests in `engine/tests/` can depend on both `mouser_engine` and `mouser_gui` (via dev-deps)
- GUI ring tests: test `RingState` transitions, geometry helpers, hit-test, layout resolution — avoid testing `draw_ring`/`handle_ring_input` which need a live egui Context
