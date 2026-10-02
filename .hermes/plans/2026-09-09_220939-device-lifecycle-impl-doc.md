# Implementation document: mouser-rs device & battery lifecycle

**Owner**: whoever is implementing the next lifecycle improvement in `mouse/`  
**Scope**: one doc, `.hermes/plans/`-adjacent artifact, not code.  
**Ceremony**: Standard (plan -> write -> validate) — this is a doc, not a feature.

---

## Goal

One reference a new contributor can read top-to-bottom in 20 minutes and then immediately know:
- what the mouser-rs device/battery pipeline looks like end-to-end,
- where each piece lives in the repo,
- which behaviors are deliberate (and why) vs. accidental (and risky),
- and what the next safe improvement to each piece is.

This doc is the *writeup* of the lifecycle, not the lifecycle itself.

---

## Audience and assumptions

- Reader has Rust basics, has used a HID mouse, has never touched this repo.
- Reader does **not** assume the OpenLogi research is authoritative — this doc cites it as comparison where useful, never as the spec.
- The active codebase is the current `mouse/` checkout on branch `new_interagation`, HEAD `c07053a`.
- All file paths are relative to `mouse/` workspace root.
- Linux-only for device/battery details (the only platform with real HID++ + sysfs + evdev + BlueZ in this tree). Cross-platform notes are marked.

---

## How to read this doc

Sections are ordered by dependency, not by importance.

1. **Polling backbone** — the thread that drives nearly everything. Read before any other section.
2. **Device inventory** — receivers + Bluetooth cache, the paired-device list the worker builds.
3. **Battery** — the two-tier read path (HID++ then sysfs), the worker wiring, and what is missing.
4. **HID++ battery decoding** — the protocol side (feature probe -> decode -> labels).
5. **User workflow** — how the GUI + CLI + tray surface the above to a human.
6. **Comparison to OpenLogi** — only for the reader who wants to see where mouser-rs already mirrors OpenLogi and where it diverges.
7. **Open gaps and next safe moves** — concrete, scoped, low-risk.

---

## 1. Polling backbone (`engine/src/worker.rs`)

### What it is

A single background thread (`spawn_background_worker`) owns the device pipeline. It:
- maintains an in-memory paired-device cache,
- polls receivers + Bluetooth periodically,
- queries battery,
- pushes a `DeviceStateUpdate` to the GUI over a channel,
- calls `repaint_callback()` to wake the egui loop.

This is the **only** thing polling devices. There is no separate battery thread and no separate receiver thread; the worker owns all of it. That is the most important architectural fact in this codebase for lifecycle understanding.

### Polling cadence

```rust
// engine/src/worker.rs — polling interval
let poll_interval = if is_connected {
    Duration::from_secs(5)
} else {
    Duration::from_secs(2) // Polling rate of 2 seconds when disconnected
};
```

- When any paired device is connected: **5 s**.
- When nothing is connected: **200 ms command drain + 2 s interval**.
- A `TriggerPoll` command from the GUI sets `force_poll = true` and fires on the next loop iteration regardless of cadence.

### Command channel

```rust
// engine/src/worker.rs
pub enum BackgroundTxCmd {
    TriggerPoll,
    Unpair(String),
}
```

Two commands today: force a poll, and Bluetooth unpair. The unpair flow has a deliberate one-cycle grace (see Section 2).

### What the worker does on each poll

In order:
1. drain `rx_cmd.recv_timeout(200ms)` — either `TriggerPoll`, `Unpair(mac)`, or timeout/exit,
2. detect receivers via `crate::receiver::detect_receivers()` -> `(unifying, bolt)` bools,
3. refresh Bluetooth paired devices via `crate::bluetooth::get_paired_logitech_devices()` -> `Vec<(mac, name, connected)>` + `bt_up`,
4. reconcile the in-memory cache against the fresh Bluetooth list (add, update, prune — see Section 2),
5. query battery (Section 3),
6. build `DeviceStateUpdate`,
7. `tx_state.send(update)` + `repaint_callback()`.

### What to verify

```bash
cd /home/soulr27/project/github_profile/linux/mouse
rg -n "spawn_background_worker|DeviceStateUpdate|BackgroundTxCmd|recv_timeout|repaint_callback" engine/src/worker.rs
cargo check -p mouser_engine
```

Expected: one `spawn_background_worker` definition, one `DeviceStateUpdate` struct, one `BackgroundTxCmd` enum, `recv_timeout(Duration::from_millis(200))` appears once, `repaint_callback()` appears once per poll cycle. `cargo check` passes.

---

## 2. Device inventory: receivers + Bluetooth cache (`engine/src/worker.rs`, `engine/src/cache.rs`, `engine/src/receiver.rs`, `engine/src/bluetooth.rs`)

### Receivers

`crate::receiver::detect_receivers()` returns `(bool, bool)` — unifying present, bolt present. Read the function for the exact detection logic and the device paths it scans.

```bash
rg -n "pub fn detect_receivers" -A 40 engine/src/receiver.rs
```

### Bluetooth paired-device cache

State: `Vec<(mac: String, name: String, connected: bool)>`, persisted to disk by `crate::cache`.

Reconciliation on each poll, in priority order:

1. Build a fresh map `fresh_map: HashMap<mac, (name, connected)>` from `get_paired_logitech_devices()`.
2. Walk the cached list **in place**:
   - if mac is in fresh_map: update name if different, update connected if different,
   - if mac is **absent** AND already `connected == false`: **remove** (prune) — device has been gone for at least one full poll cycle,
   - if mac is **absent** AND still `connected == true`: set to `false` but **keep for one more cycle** (the one-cycle grace),
   - do not increment index on remove (next element shifts in).
3. Append any brand-new macs from fresh_map not already in cache.

### Why the one-cycle grace exists

A device unpaired at the BlueZ level may still appear in `paired_devices` on the next poll because the OS hasn't fully settled. Removing immediately would make a re-pair later collide with a stale local cache entry and produce duplicate GUI cards. Keeping a disconnected-but-just-removed device for one extra cycle is the cheap fix.

> **Deliberate, not accidental.** The comment in `worker.rs` explicitly explains the tradeoff. If someone refactors this loop, preserve the grace; if they remove it, they should have a reason and update this doc.

### Unpair command

On `BackgroundTxCmd::Unpair(mac)`:
- call `crate::bluetooth::unpair_device(&mac)`;
- if it succeeds: remove from in-memory cache, **and** prune the on-disk cache so a restart can't resurrect it,
- if it fails: keep the local entry and log a warning (don't delete on a failed remove).

```bash
rg -n "BackgroundTxCmd::Unpair|unpair_device\(" engine/src/worker.rs
```

### What to verify

```bash
rg -n "detect_receivers|get_paired_logitech_devices|save_device_cache|load_device_cache|unpair_device" engine/src
cargo check -p mouser_engine
```

Expected: receiver detection in `engine/src/receiver.rs`, BlueZ paired-device refresh in `engine/src/bluetooth.rs`, cache I/O in `engine/src/cache.rs`, unpair in both `worker.rs` and `bluetooth.rs`. `cargo check` passes.

---

## 3. Battery read path (the two-tier design)

### High-level

Battery is queried inside the worker poll, **after** the device reconciliation step, in this priority:

```rust
// engine/src/worker.rs — battery priority
let has_active_hidpp = crate::battery::has_active_hidpp_battery();
let (battery_pct, battery_status) =
    if paired_devices.iter().any(|(_, _, c)| *c) || has_active_hidpp {
        match crate::battery::get_mouse_battery_hidpp() {
            Some((pct, status)) => (pct, status),
            None => match crate::battery::get_mouse_battery() {
                Some((_, pct)) => (pct, String::new()),
                None => ("80".to_string(), String::new()),
            },
        }
    } else {
        ("0".to_string(), String::new())
    };
```

Priority:
1. HID++ direct read (`get_mouse_battery_hidpp`) when a paired device exists or a HID++ battery node is present,
2. sysfs fallback (`get_mouse_battery`) when HID++ fails,
3. hard-coded `(80, "")` fallback when neither works.

If nothing is connected: `(0, "")`.

### Tier 1 — HID++ (`engine/src/battery.rs::get_mouse_battery_hidpp`)

- Opens the first reachable HID++ device via `HidppClient::open_for_battery()`,
- calls `client.read_battery()`,
- returns `Option<(String, String)>` = `(percentage, status)`.

The comment in `battery.rs` calls this *"OpenLogi approach — richer status"*. That is accurate: it reads the actual HID++ battery feature rather than relying on the kernel's sysfs interpretation.

### Tier 2 — sysfs (`engine/src/battery.rs::get_mouse_battery`)

Reads `/sys/class/power_supply` entries whose name starts with `hidpp_battery_`, for each entry:
1. `capacity` (the percentage file) — if present and non-empty, return `(cap + "%", status)`,
2. `capacity_level` (the coarse level string) — if present and non-empty, map to a percentage string via the inline map:
   - `full` -> `100`, `high` -> `80`, `normal` -> `50`, `low` -> `20`, `critical` -> `5`, else `80`,
   - returns the **original level string** as the first element and the mapped numeric string as the second,
3. status is derived from the power-supply name:
   - `AC` or `ACAD` -> `charging`,
   - name starts with `USB` or `wireless` -> `charging`,
   - `Battery` -> `discharging`,
   - otherwise: read `online` attribute; `1` -> `charging`, else `discharging`.

**Important:** the sysfs path also has a **mock path for non-Linux**: `Some(("Full".to_string(), "100".to_string()))`. This is the cross-platform default and should be treated as a placeholder, not a real reading.

### Tier 1/Nil guard: `has_active_hidpp_battery()`

Scans `/sys/class/power_supply` for any entry starting with `hidpp_battery_`. Returns `bool`. Used by the worker as a cheap gate: if sysfs says a HID++ battery node exists, the worker will *try* the HID++ path even if no paired BT device is currently connected.

### What is missing (cold start)

The code has **no** handling for the common real-world case: a device that reports `0%` while charging on a cold start until a later poll. In particular:
- sysfs `capacity == "0"` with a charging status is indistinguishable from a genuinely empty battery here,
- HID++ `parse_unified_battery` returns whatever the firmware sends (clamped to 100 at the top, but 0 is left as 0),
- the worker maps `None` -> `("80", "")` which hides a real failure but does **not** distinguish cold-start-zero from "no battery".

The OpenLogi rule *"charging + 0% = cold start, don't render as empty"* is **not** implemented.

### What to verify

```bash
rg -n "get_mouse_battery_hidpp|get_mouse_battery|has_active_hidpp_battery|open_for_battery|read_battery" engine/src/battery.rs engine/src/hidpp
rg -n "pub fn get_mouse_battery" -A 80 engine/src/battery.rs
cargo check -p mouser_engine
```

Expected: `get_mouse_battery_hidpp` and `get_mouse_battery` in `battery.rs`, `has_active_hidpp_battery` in `battery.rs`, `open_for_battery` + `read_battery` on `HidppClient` in `engine/src/hidpp/device.rs`, decode + labels in `engine/src/hidpp/protocol.rs`. `cargo check` passes.

---

## 4. HID++ battery decoding (`engine/src/hidpp/mod.rs`, `engine/src/hidpp/device.rs`, `engine/src/hidpp/protocol.rs`, `engine/src/hidpp/tests.rs`)

### Client fields

`HidppClient` (in `engine/src/hidpp/mod.rs`) holds three separate battery feature indexes:

```rust
pub(crate) battery_idx: Option<u8>,           // 0x1004 unified
pub(crate) battery_legacy_idx: Option<u8>,    // 0x1000 legacy
pub(crate) battery_voltage_idx: Option<u8>,   // 0x1001 voltage
```

Priority is unified -> legacy -> voltage, matching the OpenLogi preference order.

### Feature probe order

In `open_for_battery()` / the device connect path (`engine/src/hidpp/device.rs`):
1. probe `FEAT_UNIFIED_BATTERY` (0x1004); if found, store `battery_idx`,
2. else probe `FEAT_BATTERY_STATUS` (0x1000); if found, store `battery_legacy_idx`,
3. else probe `FEAT_BATTERY_VOLTAGE` (0x1001); if found, store `battery_voltage_idx`,
4. if none found: return error `"No battery feature (0x1004/0x1000/0x1001) found on this device"`.

### Decode functions

In `engine/src/hidpp/protocol.rs`:

```rust
pub fn parse_unified_battery(payload: &[u8]) -> (u8, String)
pub fn unified_battery_status_label(status: u8) -> String
pub fn legacy_status_label(status: u8) -> String
```

- `parse_unified_battery`: `percentage = payload[0].min(100)`, status from `unified_battery_status_label(payload[2] ?? 0)`. Note: the level byte (payload[1]) is **ignored** by the decode — the struct has the field (`BatteryReading.level`) but the current parser does not populate it from the wire.
- `unified_battery_status_label`: maps 0..7 + unknown to human labels (`discharging`, `charging`, `charging_nearly_full`, `full`, `charging_slow`, `invalid_battery`, `thermal_error`, `charging_error`).
- `legacy_status_label`: similar for 0x1000 status bytes (`recharging`, `almost_full`, `slow_recharge`, `invalid_battery`, `thermal_error`, `other`).

### Voltage battery

The voltage feature (0x1001) **probe index is stored** (`battery_voltage_idx`) but the current decoder does **not** appear to have a public `read_battery_voltage` path wired into `read_battery()`. This is a gap to confirm by reading `engine/src/hidpp/device.rs` and `engine/src/hidpp/mod.rs` directly. If voltage-only devices (G-series style) are supposed to work, the probe must be paired with a read.

### Clamping

`parse_unified_battery` clamps percentage with `.min(100)`. There is a test for it (`test_parse_unified_battery_clamps_percentage`). Nothing clamps the bottom — a firmware reporting 0 is returned as 0.

### What to verify

```bash
rg -n "FEAT_UNIFIED_BATTERY|FEAT_BATTERY_STATUS|FEAT_BATTERY_VOLTAGE|battery_idx|battery_legacy_idx|battery_voltage_idx"
    engine/src/hidpp
cargo test -p mouser_engine -- hidpp
```

Expected: three feature constants defined, three index fields on `HidppClient`, probe order in `device.rs`, three decode/label functions in `protocol.rs`, tests for unified parse + both label tables + the clamp test. `cargo test -p mouser_engine -- hidpp` passes.

---

## 5. User workflow: how a human sees this

### GUI (egui)

The GUI is `gui/` (mouser_gui). It connects to the daemon's gRPC server and receives device state through:
- `WatchDeviceState` server-streaming RPC (pushes `DeviceStateResponse`),
- `WatchConfig` server-streaming RPC (pushes config changes),
- synchronous RPCs for reads/actions.

The battery is surfaced by whatever widget reads `DeviceStateUpdate.battery_pct` and `battery_status` from the pushed state. Find the exact widget by searching for where those fields are consumed:

```bash
rg -n "battery_pct|battery_status|has_active_hidpp_battery|DeviceStateUpdate" gui/src engine/src/client.rs
```

### CLI (`engine/src/client.rs`)

`EngineClient` is the synchronous gRPC wrapper the CLI uses. It mirrors the daemon's `Engine` API and reuses the same proto stubs. Battery is not currently a first-class CLI subcommand — confirm by listing CLI commands:

```bash
rg -n "Command|clap|subcommand|status|profile|dpi" engine/src/client.rs engine/src/main.rs
```

If no battery subcommand exists, that is the current state and this doc records it as such.

### Tray

The tray (with restore/quit) is not battery-aware beyond whatever the daemon pushes. Confirm by reading `engine/src/main.rs` tray path.

### Cork validation rule

Any workflow claim in this doc that names a specific widget, RPC, or UI screen must be backed by a grep that finds it. If the grep is empty, the claim is removed or marked "to confirm".

---

## 6. Comparison to OpenLogi (context only, not a spec)

For a reader who has seen the OpenLogi deep-dive, here is where mouser-rs matches and where it differs.

| Concern | OpenLogi | mouser-rs today |
|---|---|---|
| Polling model | event-first, reconciliation triggers with settle windows + 30 s recovery scan | fixed-interval thread, 5 s connected / 2 s disconnected, no event coalescing |
| Battery priority | 0x1004 -> 0x1000 -> 0x1001 (protocol), same as mouser-rs feature probe order | same priority order in probe |
| Battery decoding | 0x1004 percentage+level+status, 0x1000 discharge_level/next_level/status, 0x1001 voltage->percent curve | 0x1004 parse (percentage, status; level byte unused), 0x1000 status label only, 0x1001 probe stored but read path unclear |
| Cold-start 0% | guarded (`charging && 0%` != empty) | not guarded |
| Voltage->percent curve | 13-point Solaar Li-Po curve, linear interpolation | not present (voltage tier may be probe-only) |
| Node health ledger | replay last-good inventory for N ticks, evict/reopen channel after M | no ledger; worker relies on poll cadence + Bluetooth grace |
| Paired-device cache | receiver registers + BT marketing name, offline devices surfaced | in-memory `Vec<(mac,name,connected)>` + disk cache, Unifying/Bolt as bools, no per-device HID++ slot identity visible in the cache |
| IPC | tarpc/bincode, level-triggered `observe(generation)`, full-state on change | tonic gRPC, server-streaming watch RPCs + sync stubs |
| Pairing | Bolt discovery->passkey (keyboard/pointer clicks), Unifying lock | not in mouser-rs battery/device scope today (BlueZ unpair only) |

Reads:
- mouser-rs worker: `engine/src/worker.rs`
- mouser-rs battery: `engine/src/battery.rs`
- mouser-rs hidpp decode: `engine/src/hidpp/protocol.rs`
- mouser-rs hidpp client + probe: `engine/src/hidpp/mod.rs`, `engine/src/hidpp/device.rs`

---

## 7. Open gaps and the next safe move for each

### 7.1 Cold-start 0% while charging

**Gap:** no rule distinguishing a charging device reporting 0% from a genuinely empty battery.

**Cheapest safe fix:** in `engine/src/battery.rs`, when sysfs returns `capacity == "0"` AND status is charging, return a sentinel like `(percentage = 0, status = "charging_cold")` or preserve the charging status and let the GUI treat 0+charging as "unknown until later". This is the OpenLogi rule in mouser-rs terms.

**Scope:** one function, `get_mouse_battery`. No protocol change, no worker change.

**Verification:**
```bash
# after the change:
cargo check -p mouser_engine
# add a unit test in battery.rs:
#   assert cold-start path when capacity == "0" and status is charging
cargo test -p mouser_engine -- battery
```

### 7.2 Voltage battery read path

**Gap:** `battery_voltage_idx` is stored but it is unclear whether a `read_battery_voltage()` exists and is wired. If it is not, voltage-only devices get no percentage.

**Safe next move:** read `engine/src/hidpp/device.rs` and `engine/src/hidpp/mod.rs` to confirm. If there is no read path, the gap is "voltage tier probe-only". The fix is out of scope for this doc unless the reader wants to implement the Solaar-style voltage->percent curve.

### 7.3 Level byte unused in unified decode

**Gap:** `parse_unified_battery` ignores `payload[1]` (the `BatteryLevel` bucket), but `BatteryReading.level` exists as a field.

**Safe next move:** populate `level` from `payload[1]` using a level-label match, update `BatteryReading` usage sites, add a test. Small, contained, testable.

### 7.4 Polling model

**Gap:** fixed 5 s / 2 s polling, no event coalescing, no recovery scan. This is architectural and not small.

**Safe next move (if wanted):** introduce a reconciliation-trigger enum behind the existing worker loop without changing the external behavior first — e.g., a `ReconcileTrigger` that the existing 5 s / 2 s timer and `TriggerPoll` command map onto, plus a future 30 s recovery arm. Do this as a separate doc + feature, not folded into the battery doc.

---

## 8. Doc source-of-truth checklist

Before this doc ships, each claim in it must be backed by at least one of:
- a direct file read,
- a grep that returns hits,
- a `cargo check`/`cargo test` run that passes.

Commands to run as a final validation pass:

```bash
cd /home/soulr27/project/github_profile/linux/mouse
set -e
rg -n "spawn_background_worker|DeviceStateUpdate|BackgroundTxCmd|recv_timeout|repaint_callback" engine/src/worker.rs
rg -n "detect_receivers|get_paired_logitech_devices|save_device_cache|load_device_cache|unpair_device" engine/src
rg -n "get_mouse_battery_hidpp|get_mouse_battery|has_active_hidpp_battery" engine/src/battery.rs
rg -n "parse_unified_battery|battery_idx|battery_legacy_idx|battery_voltage_idx|FEAT_UNIFIED_BATTERY|FEAT_BATTERY_STATUS|FEAT_BATTERY_VOLTAGE" engine/src/hidpp
rg -n "battery_pct|battery_status|DeviceStateUpdate" gui/src engine/src/client.rs engine/src/grpc
cargo check -p mouser_engine
cargo test -p mouser_engine -- hidpp
echo "ALL CHECKS PASSED"
```

---

## 9. What this doc is NOT

- It is not a feature spec. It does not authorize any code change.
- It is not a security or safety review.
- It is not a cross-platform battery spec — it records the Linux path and the non-Linux mock default, and marks both honestly.

---

## Appendix A. Files this doc references

| File | Role |
|---|---|
| `engine/src/worker.rs` | polling backbone + device reconciliation + battery wiring |
| `engine/src/battery.rs` | two-tier battery read (HID++ + sysfs) + cold-start gap |
| `engine/src/hidpp/mod.rs` | `HidppClient`, `BatteryReading`, feature index fields |
| `engine/src/hidpp/device.rs` | feature probe order, `open_for_battery`, `read_battery` |
| `engine/src/hidpp/protocol.rs` | `parse_unified_battery`, status/label functions, clamp |
| `engine/src/hidpp/tests.rs` | battery decode + label tests |
| `engine/src/cache.rs` | paired-device on-disk cache |
| `engine/src/receiver.rs` | Unifying/Bolt receiver detection |
| `engine/src/bluetooth.rs` | BlueZ paired-device refresh + unpair |
| `engine/src/client.rs` | synchronous gRPC `EngineClient` used by CLI/GUI |
| `engine/src/grpc/server.rs` | tonic UDS server + watch streams |

---

## Changelog

- 2026-09-09 — initial version, covering polling backbone, device cache, two-tier battery, HID++ decode, user workflow, OpenLogi comparison, and open gaps.
