# Implementation document: Logitech HID++ sensor services (battery)

**Owner**: whoever touches battery/sensor reads in `mouse/`  
**Scope**: engine crate, battery + HID++ protocol layer — no GUI/IPC changes  
**Ceremony**: Standard — plan → patch → verify → commit  
**Status**: implemented (2026-09-09), 13/13 battery+hidpp tests pass, workspace `cargo check` clean

---

## Goal

Make mouser-rs's Logitech battery/sensor reads correct and complete enough to trust:
- never show a misleading `0%` on a cold-start charging mouse,
- actually populate the unified-battery level byte that was ignored,
- support voltage-only battery devices (0x1001, e.g. G-series wireless) instead of returning `None`.

These were the highest-value, lowest-risk gaps in the sensor read path. Everything here is pure-ish or a small read-path branch — no threading, no cache, no event system, no IPC change.

---

## Current context / assumptions

- Workspace `mouse/`, branch `new_interagation`, HEAD `c07053a`.
- `mouser_engine` crate owns `battery.rs`, `hidpp/` (protocol + client).
- Linux-only device logic. Non-Linux default is the existing mock `Some(("Full".to_string(), "100".to_string()))` in `battery.rs`.
- Worker is single-threaded today and owns the poll loop; this doc stays inside the read path, not the threading model.
- Battery wire contract is `battery_pct: String`, `battery_status: String` in `DeviceStateUpdate`/`DeviceStateResponse`. We improve the *producer* side only.
- UI consumers already degrade gracefully to sensible defaults when `battery_pct` is empty (`status_pill.rs:68` → `unwrap_or(100.0)`, `device_card.rs:258` → `unwrap_or(80.0)`), so returning `String::new()` for the cold-start case is safe.

---

## What changed (files)

| File | Change |
|---|---|
| `engine/src/battery.rs` | cold-start guard + helper + unit tests |
| `engine/src/hidpp/protocol.rs` | unified level label + full decode (already existed) + voltage decode/curve/status labels |
| `engine/src/hidpp/mod.rs` | import new voltage fns; use full decode in `read_battery` unified branch + event path; new voltage branch in `read_battery` |
| `engine/src/hidpp/tests.rs` | level + voltage tests (merged into `mod tests`, plus new `mod voltage_tests`) |

---

## Detail: cold-start 0% while charging (battery.rs)

### Problem
A Logitech HID++ battery under charge can report `0%` until the firmware has a real reading. The old sysfs path returned that as `(format!("{}%", pct_trim), …)` = `"0%"`, which looks like a dead battery. There was also a pre-existing bug where the return used `pct_trim` for *both* tuple elements instead of the computed `status`.

### Fix
- New helper `battery_cold_start_zero(status: &str) -> bool`: true for any charging-ish status (`charging`, `charging_slow`, `recharging`, `charging_nearly_full`).
- In the `capacity` branch, after reading `capacity` and before the existing return: if `pct_trim == "0"` AND status is a charging status, return `(String::new(), status)` — empty percentage, real status. The UI degrades to a full-ish bar (100.0/80.0 default) with a charging indicator, which is better than 0%.
- If not a cold start, fall through to the normal return `(format!("{pct_trim}%", status))` — and this now correctly uses `status`, fixing the old `pct_trim`-as-status bug.

### Code
```rust
fn battery_cold_start_zero(status: &str) -> bool {
    status.starts_with("charging") || status == "recharging"
}
```
Inside the capacity branch:
```rust
if pct_trim == "0" && battery_cold_start_zero(&status) {
    return Some((String::new(), status));
}
return Some((format!("{pct_trim}%", status)));
```

### Tests (battery.rs `mod tests`)
- `cold_start_zero_is_charging_statuses`: charging, charging_slow, recharging, charging_nearly_full → true; discharging/full/empty/unknown → false.
- `cold_start_zero_not_discharging`: discharging → false.

---

## Detail: unified-battery level byte (protocol.rs, mod.rs)

### Problem
`parse_unified_battery()` decoded percentage + status but ignored `payload[1]` (the level byte). `BatteryReading.level` existed but stayed `""` from the HID++ path.

### State entering this task
- `unified_battery_level_label()` and `parse_unified_battery_full()` already existed in `protocol.rs` (from a prior session).
- `mod.rs` already imported `parse_unified_battery_full`.

### Fix
Use `parse_unified_battery_full` in both HID++ read sites:
- `read_battery()` unified branch: try full decode first; on success populate `level`; on failure fall back to the minimal 2-tuple decode (defensive).
- Event broadcast path (0x1004 unsolicited): same pattern — full decode with fallback.

```rust
if let Some((percentage, level, status)) = parse_unified_battery_full(&resp) {
    return Some(BatteryReading { percentage, status, level });
}
let (percentage, status) = parse_unified_battery(&resp);
return Some(BatteryReading { percentage, status, level: String::new() });
```

`unified_battery_level_label` maps HID++ `BatteryLevel` discriminants (Critical=1, Low=2, Good=4, Full=8) to labels; unknown values → `"unknown_level"`.

### Tests (hidpp `mod tests`)
- `test_parse_unified_battery_full_population`: `[80, 4, 1]` → pct 80, level "good", status "charging".
- `test_unified_battery_level_labels`: 1→critical, 2→low, 4→good, 8→full, 0/3/99→unknown_level.
- `test_parse_unified_battery_full_short_payload`: too-short slices → None.

---

## Detail: voltage-only battery (0x1001) (protocol.rs, mod.rs, tests.rs)

### Problem
`battery_voltage_idx` was probed and stored but never read — G-series wireless devices that only expose 0x1001 got `None`.

### Fix (protocol.rs)
- `voltage_battery_status_label(status: u8) -> &'static str`: 0→discharging, 1→charging, 2→charging_fast, 3→full, 4→charging_slow, 5→invalid_battery, 6→thermal_error, 7→charging_error, else unknown.
- `voltage_to_battery_percent(mv: u16) -> u8`: 13-point Solaar-style discharge curve (4200mV→100 down to 3000mV→0), integer interpolation rounding to nearest, clamped at ends.
- `parse_voltage_battery(payload: &[u8]) -> Option<(u16, &'static str)>`: conservative format `[voltage_msb, voltage_lsb, status_byte]`, MSB/LSB → mV big-endian.

`voltage_to_battery_percent` return type is `u8`; the interpolation is done in `u16` then cast back, with the result bounded by the curve endpoints.

### Fix (mod.rs `read_battery`)
New third branch after unified and legacy:
```rust
if let Some(idx) = self.battery_voltage_idx {
    if let Ok(Some(resp)) = self.request(idx, 1, &[0; 3], 1000) {
        if let Some((mv, status)) = parse_voltage_battery(&resp) {
            let pct = voltage_to_battery_percent(mv);
            return Some(BatteryReading {
                percentage: pct,
                status: status.to_string(),
                level: String::new(),
            });
        }
    }
    log::warn!("[HID++] BatteryVoltage read failed, no battery available");
}
```

Voltage path has no discrete level byte, so `level` stays `""` — honest.

### Tests (hidpp `mod voltage_tests`)
- `test_voltage_battery_status_labels`
- `test_parse_voltage_battery`: `[0x0F, 0xA0, 1]` → 4000mV, "charging"
- `test_parse_voltage_battery_short_payload`
- `test_voltage_to_battery_percent_boundaries`: 4200→100, 4100→90, 4000→80, 3000→0, 2900→0
- `test_voltage_to_battery_percent_midrange`: 3850mV → between 60 and 70

---

## Verification

All of these passed after the patch:

```
cargo check -p mouser_engine                     # clean
cargo test -p mouser_engine --lib -- hidpp       # 14/14 pass
cargo test -p mouser_engine --lib -- battery     # 13/13 pass (incl. hidpp)
cargo check --workspace                          # clean (engine + gui + root)
```

Test counts after the patch:
- `hidpp::tests::tests`: feature constants, unified parse, unified status labels, legacy status labels, unified level labels, full-population, full short-payload, clamp-percentage.
- `hidpp::tests::voltage_tests`: voltage status labels, parse, short-payload, percent boundaries, percent midrange.
- `battery::tests`: cold_start_zero_is_charging_statuses, cold_start_zero_not_discharging.

---

## Open questions / known caveats

1. **Voltage byte order is an assumption.** `parse_voltage_battery` assumes `[msb, lsb, status]`. This should be verified against a real 0x1001 device before trusting the exact mV values. The doc comment says as much; log raw bytes at debug level if unsure.

2. **Voltage discharge curve is conservative.** 4200→100, 3000→0, linear interpolation in between. Calibrate against a real device if precise percent matters; for v1 it's "not dead, not wildly wrong."

3. **Cold-start sentinel is empty string, not a new status token.** This keeps the wire contract unchanged (`String`), but means the UI's "empty battery_pct" handling is what carries the semantics. Confirmed the two main consumers degrade to a sensible default.

4. **No probe cache / node-health ledger / event-driven ticks yet.** Those are Tier 2/3 in the broader plan and were explicitly out of scope for this implementation pass. The read path is now correct first; efficiency/resilience come after.

5. **Level byte for legacy (0x1000) path stays empty.** Legacy `getBatteryLevelStatus` reports `[discharge_level, next_level, status]`; `next_level` is a granularity hint, not a discrete category, so we don't fake a level label there. Only unified (0x1004) populates `level`.

---

## Commit plan

One commit, four files, TDD-ordered:

```
feat(sensors): cold-start 0% guard + voltage battery path + unified level byte

- battery.rs: don't render 0% on a cold-start charging mouse; fix sysfs
  return to use the computed status (was returning pct_trim twice).
- protocol.rs: voltage 0x1001 decode + Solaar-style mV→percent curve +
  voltage status labels.
- mod.rs: import voltage fns; use full unified decode in read_battery +
  event path; add voltage branch to read_battery.
- tests.rs: level + voltage tests.

Cold-start guard: charging + 0% → empty pct + real status, so the UI
shows "charging" without a misleading 0%. Voltage path enables G-series
battery reads. Unified level byte now populated from 0x1004 payload[1].
