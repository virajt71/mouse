# Mouser-RS Audit Report

This report summarizes the issues found in the Mouser-RS repository, categorized by severity and type.

---

## 1. Critical Issues (Potential Crashes & Deadlocks)

### 1.1 Deadlock Risk in Clipboard Synchronization
**File:** `engine/src/flow/clipboard.rs`
**Description:** There is a classic lock-order inversion between `LAST_IMG_SIG` and `LAST_IMG_HASH`.
- In `run_clipboard_loop()`, `LAST_IMG_SIG` is locked first, then `LAST_IMG_HASH`.
- In `set_local_clipboard_image()`, `LAST_IMG_HASH` is locked first, then `LAST_IMG_SIG`.
**Impact:** If both threads attempt to update the image clipboard simultaneously, the application will deadlock.
**Recommendation:** Always acquire locks in the same order across the entire codebase.

### 1.2 GUI Crash on Engine Panic
**File:** `gui/src/` (various)
**Description:** The GUI heavily uses `.lock().unwrap()` on Mutexes shared with the engine.
**Impact:** If any background thread in the engine panics while holding a lock (e.g., during device disconnection or network failure), the Mutex becomes "poisoned." Subsequent attempts by the GUI to lock it will also panic, crashing the entire user interface.
**Recommendation:** Use `lock().unwrap_or_else(|e| e.into_inner())` or handle poisoned locks gracefully.

### 1.3 Panics in Library Code
**File:** `engine/src/flow/network.rs`, `engine/src/input/simulator/hangul.rs`
**Description:** Several `panic!()` calls exist in what should be robust library code.
**Impact:** Unexpected network packets or internal state in the Hangul simulator will crash the entire daemon.
**Recommendation:** Replace `panic!` with `Result` and handle errors at the caller level.

---

## 2. Major Issues (Security & Logic)

### 2.1 Overly Broad udev Rules
**File:** `packaging/linux/69-mouser-logitech.rules`
**Description:** The rule `KERNEL=="event*", SUBSYSTEM=="input", ATTRS{idVendor}=="046d", TAG+="uaccess"` grants the desktop user access to *all* Logitech input events.
**Impact:** While intended for mice, this also gives the user raw access to Logitech keyboards, potentially allowing a malicious user-space process to sniff keystrokes (keylogging) without root privileges.
**Recommendation:** Narrow the scope to specific product IDs or use more specific attributes to target only mouse buttons.

### 2.2 Unix Socket Security & Persistence
**File:** `src/single_instance.rs`, `src/gui.rs`
**Description:**
- The Unix socket `mouser.sock` is created with default permissions (usually 0775 or 0755), which might be too permissive depending on the system's umask.
- Message processing is minimal (`String::from_utf8_lossy(&buf[..n])`) and only checks for `"SHOW"`.
- If the app crashes, the socket file is not removed, which can lead to "Address already in use" errors on restart (though there is some stale socket cleanup logic, it's not foolproof).
**Recommendation:** Explicitly set socket permissions to 0600 and implement more robust message validation.

### 2.3 Fragile Device Layout Detection
**File:** `engine/src/hidpp/device.rs`
**Description:** `get_layout_key_from_name` uses case-insensitive string matching on the product string.
**Impact:** Many Logitech devices will fall back to "generic," resulting in missing customization options for buttons that exist but aren't explicitly named (e.g., "MX Master 3" vs "MX Master 3S").
**Recommendation:** Use USB Product IDs (PIDs) for reliable device identification instead of string matching.

---

## 3. Minor Issues (Build & Environment)

### 3.1 Incomplete Dependency Documentation
**File:** `README.md`
**Description:** The README lists some dependencies but misses others like `libglib2.0-dev` and `libgtk-3-dev` (though `libgtk-3-dev` is mentioned in the `apt install` command, it might be missed by users of other distros).
**Impact:** Users on fresh installs will encounter "Package gobject-2.0 not found" errors during compilation.
**Recommendation:** Provide a comprehensive list of system dependencies or a setup script for major distributions.

### 3.2 Protocol Implementation TODOs
**File:** `engine/src/hidpp/mod.rs`
**Description:** Several critical offsets and function IDs in the HID++ protocol handling are marked with `TODO`.
**Impact:** Backlight state change notifications might be misparsed or ignored on certain devices until these are confirmed.
**Recommendation:** Verify these values against official HID++ documentation or packet dumps and finalize the implementation.

---

## 4. Documentation & Maintenance

### 4.1 Contradictory License
**File:** `README.md`
**Description:** The README states the project is "private / proprietary," but it lacks a formal LICENSE file and is being shared for review.
**Impact:** Legal ambiguity regarding the right to use, modify, or distribute the code.
**Recommendation:** Include a standard LICENSE file (e.g., MIT, GPL, or a clear Proprietary license).

### 4.2 Low Test Coverage
**Description:** The project has very few unit tests and only one integration test file.
**Impact:** High risk of regressions during refactoring, especially in the complex gesture and event-loop logic.
**Recommendation:** Increase test coverage for the `engine` logic, particularly for protocol parsing and gesture state machine.
