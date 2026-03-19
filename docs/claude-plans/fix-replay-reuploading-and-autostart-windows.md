# Fix replay re-uploading and autostart persistence on Windows

## Context

Two bugs on Windows:
1. Every app restart triggers re-uploading of all replays because the SHA256 dedup check is coupled to the 100-entry display history — files beyond that cap lose their dedup records
2. The "Launch at login" toggle doesn't persist because a `$effect` in SettingsView queries `isEnabled()` on mount and overrides the toggle if the OS returns `false`

## Bug 1: Decouple SHA256 dedup from display history

**Root cause**: `has_sha256()` in `state.rs` iterates `uploads` (capped at 100). Users with >100 replays lose old entries, causing re-uploads every startup.

**Fix**: Add a separate `known_hashes: HashSet<String>` to `AppState`, persisted independently.

### `src-tauri/src/state.rs`
- Add `known_hashes: HashSet<String>` field to `AppState`
- Change `has_sha256()` to check `self.known_hashes.contains(sha256)`
- In `add_entry()`, also insert the entry's SHA256 into `known_hashes`

### `src-tauri/src/config.rs`
- Add `load_known_hashes()` -> reads `"knownHashes"` key from store into `HashSet<String>`
- Add `save_known_hashes()` -> writes the set to `"knownHashes"` key

### `src-tauri/src/lib.rs`
- Load `known_hashes` from store at startup
- Seed it from existing history entries (migration for first launch after update)

### `src-tauri/src/watcher.rs`
- In `persist_and_emit()`: also save `known_hashes` to store
- In `rescan()`: also clear `known_hashes` so "Re-upload All Replays" works

## Bug 2: Autostart doesn't work on Windows

**Root causes**:
1. The `auto-launch` crate (used by `tauri-plugin-autostart`) doesn't quote the exe path in the Windows registry entry. With `productName = "Storm Uploader"`, the installed path contains spaces and gets misinterpreted by Windows at startup.
2. The `$effect` in SettingsView overrides the toggle state on every mount, and there's no error handling around `enable()`/`disable()`.

**Fix**: Bypass the plugin on Windows with custom Tauri commands that properly quote the registry path. Keep the plugin for macOS.

### `src-tauri/Cargo.toml`
- Add `winreg` as a Windows-only dependency

### `src-tauri/src/autostart.rs` (new file)
- Implement `enable_autostart`, `disable_autostart`, `is_autostart_enabled` Tauri commands
- On Windows: use `winreg` directly to read/write `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` with a properly quoted exe path
- On macOS: delegate to the plugin's `AutoLaunchManager` via `tauri_plugin_autostart::ManagerExt`

### `src-tauri/src/lib.rs`
- Register the new autostart commands in `invoke_handler`

### `src/lib/SettingsView.svelte`
- Replace `@tauri-apps/plugin-autostart` imports with `invoke()` calls to custom commands
- Replace `$effect` with `onMount` for initial state loading
- Add `autostartLoaded` state -- disable toggle until initial check resolves
- Add `autostartError` state for user-visible error feedback
- Wrap enable/disable in try/catch with error display
