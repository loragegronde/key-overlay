# Key Overlay

Lightweight **keyboard / mouse / controller overlay** for streamers and speedrunners.

Pure **Rust** desktop app (`eframe` / `egui`) — no Tauri, no WebView, no Node in production.

## Features

- Editor window to build layouts (drag, resize, rebind, undo/redo)
- Transparent always-on-top HUD with click-through lock
- Multi-layout library (create / duplicate / rename / switch)
- Keyboard + mouse capture (`rdev`) and gamepad sticks/buttons (`gilrs`)
- Themes, press effects, KPS meter, target-app visibility filter
- Persists to `layout.json` (v4 library schema)

## Hotkeys

| Shortcut | Action |
| --- | --- |
| `Ctrl+Shift+O` | Show / hide overlay |
| `Ctrl+Shift+L` | Lock overlay (click-through) after placing |
| `Ctrl+Shift+E` | Focus editor (Windows) |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo in the editor |

## Build

### Windows

```powershell
.\dev.ps1 -Release
.\dev.ps1 -Release -Run
```

`dev.ps1` loads the MSVC toolchain (`vcvars64`) before calling Cargo.

### Linux / macOS

```bash
cargo build --release
cargo run --release
```

Global hotkeys + tray are fully wired on Windows. Linux/macOS still run the editor/HUD; foreground app-filter is a stub.

## Layout file

Saved under:

- Windows: `%APPDATA%\com.keyoverlay.app\layout.json`
- Linux: `~/.config/key-overlay/layout.json` (via `directories`)

Existing v1–v3 single profiles and v4 libraries from the previous Tauri build are migrated on load.

## Place overlay

1. Arrange keys in the editor  
2. Click **Place Overlay**  
3. Drag the HUD anywhere  
4. Press **Ctrl+Shift+L** to lock click-through  
