# Key Overlay

A customizable keyboard and mouse input overlay HUD, built with Tauri v2, React, TypeScript and Tailwind. It draws a transparent, always-on-top window that lights up keys as you press them — for streaming, recording, or watching your own inputs.

Global input is captured in Rust with `rdev`, so the overlay reacts to keys pressed in **any** application, not just its own window.

---

## Contents

- [Features](#features)
- [Hotkeys](#hotkeys)
- [Modes](#modes)
- [Runtime requirements](#runtime-requirements-read-this-first)
- [Building](#building)
  - [Windows](#windows)
  - [Linux](#linux)
  - [macOS](#macos)
- [Distribution artifacts](#distribution-artifacts)
- [Troubleshooting](#troubleshooting)
- [Profile storage and schema](#profile-storage-and-schema)
- [Architecture](#architecture)
- [Known limitations](#known-limitations)

---

## Features

- **Transparent, frameless, always-on-top window** with per-window opacity
- **Global input capture** — key and mouse-button events from anywhere on the system
- **Edit mode canvas** — drag to reposition, corner handles to resize, shift+click multi-select, six-way alignment, optional grid snap, arrow-key nudge
- **Customization drawer** — per-key and global colour pickers, border radius, opacity, font size, readout toggles
- **Layout presets** — Full 100%, TKL, 60%, WASD Gaming, Arcade Stick, Streamer HUD, Custom
- **Theme presets** — Cyberpunk, Glassmorphism, Retro Arcade, Stealth Minimal, RGB Wave
- **Press animations** — Glow, Glow Pulse, Key Drop, Border Ripple, None
- **Live readouts** — per-key press counters, hold duration, and a keys-per-second meter
- **Rebindable keys** — pick a key on the canvas and press any physical key or mouse button to bind it
- **Profile export / import** to a JSON file, in addition to continuous autosave
- **Tray icon** with show and quit

## Hotkeys

| Shortcut | Action |
| --- | --- |
| `Ctrl+Shift+O` | Show / hide the overlay window |
| `Ctrl+Shift+L` | Lock (click-through) / unlock for editing |

Both are registered globally at startup by the desktop shell. They are **not** rebindable from the UI — see [Known limitations](#known-limitations).

## Modes

**Edit mode** — the window accepts input. Arrange keys, open the customization drawer, and hover the bottom edge of the screen to reveal the window control bar (lock, settings, export, minimise, exit).

**Overlay mode (locked)** — the window is click-through: every click passes to whatever is behind it. The control bar and the editing toolbar are unmounted, not merely hidden, so nothing can intercept a click. Unlock with `Ctrl+Shift+L` or the pill at the top of the screen.

---

## Runtime requirements (read this first)

These are what an **end user** needs, separate from build tooling.

### Windows

WebView2 must be present. It ships as an OS component on Windows 11 and has been distributed to Windows 10 through Windows Update since 2021, so on a normally patched machine it is already there. It is **not** guaranteed on Windows 10 LTSC, Windows Server, or images that never received the Evergreen Runtime.

To be precise about what "self-contained" means here:

| Artifact | Single file? | Runs without WebView2 installed? |
| --- | --- | --- |
| Portable `key-overlay.exe` | Yes | **No.** It will not start. |
| NSIS installer (`downloadBootstrapper`) | Yes | Yes, but it **downloads** WebView2 during install, so it needs internet. |
| NSIS installer (`offlineInstaller`) | Yes | Yes, fully offline — at roughly +130 MB. |

`webviewInstallMode` is set to `downloadBootstrapper` in `src-tauri/tauri.conf.json`, which keeps the installer small. Change it to `offlineInstaller` if you need to install on machines with no network. The setting affects **installers only** — it cannot make the portable `.exe` self-contained, because a bare executable has no install step in which to provision the runtime.

### Linux

Two shared libraries must be present at runtime:

```bash
# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1
```

`webkit2gtk` is the web renderer and `libayatana-appindicator3` backs the tray icon.

**A truly self-contained single binary is not achievable on Linux.** Tauri renders through the system WebKitGTK rather than bundling a browser engine, and webkit2gtk cannot reasonably be statically linked — it pulls in GTK, GLib, ICU, GStreamer and a long tail of system libraries. The AppImage bundles what it can, but webkit2gtk is still expected from the host in practice. Plan on the dependency, not around it.

### macOS

Global input capture needs both permissions, granted to the app (or to your terminal, when running from a dev build):

- **System Settings → Privacy & Security → Accessibility**
- **System Settings → Privacy & Security → Input Monitoring**

**`rdev` fails silently without these.** There is no error, no prompt and no log line — the overlay simply never lights up. If keys do nothing on macOS, check these two lists first.

---

## Building

Common to all platforms:

1. **Node.js** 18 or newer
2. **Rust** stable, via [rustup.rs](https://rustup.rs/)
3. `npm install` in the repository root

### Windows

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload. This provides both the MSVC toolchain and the Windows SDK.

> **The `ucrt.lib` trap.** `rustc` invokes `link.exe`, which only finds `ucrt.lib` and the rest of the Windows SDK when the **MSVC developer environment** is loaded. A plain PowerShell window does not have it, and the failure looks like a Rust problem when it is not:
>
> ```
> LINK : fatal error LNK1104: cannot open file 'ucrt.lib'
> ```
>
> This has cost real debugging time on this project. Use the helper script, which locates the newest `vcvars64.bat` (via `vswhere`, falling back to scanning the standard install roots), imports its environment, and then runs the Tauri command:
>
> ```powershell
> .\dev.ps1             # tauri dev
> .\dev.ps1 build       # tauri build, produces the NSIS installer
> .\dev.ps1 portable    # portable .exe only, skips the bundler
> ```
>
> Equivalently, run `npm run tauri dev` from the **x64 Native Tools Command Prompt for VS**. To confirm the environment loaded, check that `$env:LIB` is non-empty and contains a path ending in `ucrt\x64`.

To produce just the portable executable without invoking the bundler:

```powershell
.\dev.ps1 portable
```

That builds the frontend, then runs `cargo build --release` inside the MSVC environment and prints the resulting size. The output is `src-tauri/target/release/key-overlay.exe`. Running `cargo build --release` by hand works too, but only from a shell where the MSVC environment is already loaded — otherwise you get the `ucrt.lib` error above.

### Linux

Build dependencies (Debian / Ubuntu; adapt for other distributions):

```bash
sudo apt install \
  libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  libx11-dev libxi-dev libxtst-dev
```

The last three are for `rdev`'s X11 input hook, which is separate from Tauri's own requirements.

```bash
npm run tauri build
```

### macOS

Install the Xcode Command Line Tools (`xcode-select --install`), then:

```bash
npm run tauri build
```

> **macOS bundling is currently blocked.** `src-tauri/icons/` has no `icon.icns`, and the entry was removed from `bundle.icon` in `tauri.conf.json` rather than left dangling. Generating one from the existing 128 px PNGs would produce a visibly bad icon at Retina sizes. To unblock:
>
> 1. Provide a square source image, **1024×1024 PNG** with transparency.
> 2. Run `npx tauri icon path/to/source.png` — this regenerates every platform's icon set, including `icon.icns`.
> 3. Add `"icons/icon.icns"` back to the `bundle.icon` array in `src-tauri/tauri.conf.json`.
>
> Until then, `cargo build --release` works on macOS but `.app` / `.dmg` bundling will not.

> **`macOSPrivateApi: true` is required.** Window transparency does not work on macOS without it. It is set in both `tauri.conf.json` (`app.macOSPrivateApi`) and `Cargo.toml` (the `macos-private-api` feature on `tauri`), and the two must stay in sync. **It makes the app ineligible for the Mac App Store.** Direct distribution and notarisation are unaffected.

---

## Distribution artifacts

`bundle.targets` is set to `["nsis", "app", "dmg", "appimage", "deb"]`. The bundler ignores targets that do not apply to the host platform.

| Platform | Artifact | Notes |
| --- | --- | --- |
| Windows | `target/release/key-overlay.exe` | Single portable file. Needs WebView2 on the system. |
| Windows | NSIS `.exe` installer | Installs WebView2 if missing (needs internet at install time). |
| Linux | AppImage, `.deb` | Both still require webkit2gtk from the host. |
| macOS | `.app`, `.dmg` | Blocked on `icon.icns`, see above. |

The MSI target was dropped deliberately: it requires a WiX toolchain download and NSIS already covers the installer case for a small overlay utility. Re-add `"msi"` to `bundle.targets` if you need enterprise deployment.

The release profile in `src-tauri/Cargo.toml` is tuned for size: `opt-level = "s"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.

**Measured on this machine** (Windows 11, x86_64-pc-windows-msvc, Rust stable):

| Metric | Value |
| --- | --- |
| `key-overlay.exe` (release, stripped) | **3,412,480 bytes — 3.25 MiB** |
| Processes at runtime | 7 — one host plus six WebView2 |
| Idle working set, whole process tree | ~332 MB |
| Idle private commit, whole process tree | ~168 MB |

Memory was sampled 8 seconds after launch on the default WASD profile, counting the host process plus only the `msedgewebview2` processes that appeared after launch. Working set overstates the cost because Chromium's code pages are shared between WebView2 processes and with any other WebView2 app on the machine; **private commit (~168 MB) is the figure actually attributable to this app.**

Two honest caveats about the original "< 50 MB" target:

- It holds comfortably for the **executable** — 3.25 MiB — but only because the web renderer is not bundled. WebView2 is supplied by the OS and costs nothing in the binary.
- It does **not** hold for **memory**. A WebView app cannot idle in 50 MB; ~170 MB private is the realistic floor for anything Chromium-backed. The Rust host itself is only ~6 MB private — essentially all of the footprint is WebView2.

---

## Troubleshooting

**`LINK : fatal error LNK1104: cannot open file 'ucrt.lib'` (Windows)**
The MSVC developer environment is not loaded. Use `.\dev.ps1`. See [Windows](#windows).

**The overlay shows, but no keys light up**
- macOS: Accessibility **and** Input Monitoring permissions are missing. `rdev` gives no error.
- Linux: you are on a Wayland session. `rdev` is X11-only; see [Known limitations](#known-limitations).
- Any platform: the key you pressed may not be bound. Select the key in edit mode, open the drawer's Visuals tab and use **Rebind**.

**Clicks do not reach the window**
The overlay is locked. Press `Ctrl+Shift+L`.

**The window vanished**
`Ctrl+Shift+O` toggles visibility. The tray icon also has a Show entry.

**`layout.json` is corrupt or has odd values**
The loader validates every field and falls back per-field rather than discarding the file. Deleting it restores the default WASD profile.

---

## Profile storage and schema

One profile is stored, as JSON, at:

| Platform | Path |
| --- | --- |
| Windows | `%APPDATA%\com.keyoverlay.app\layout.json` |
| macOS | `~/Library/Application Support/com.keyoverlay.app/layout.json` |
| Linux | `~/.config/com.keyoverlay.app/layout.json` |

Edits autosave, debounced by 400 ms so that dragging a key produces one write rather than hundreds. **Export** (control bar) and **Import** (drawer → Settings) read and write arbitrary paths for backup and sharing.

**Current schema version: 2.** The file is plain JSON and safe to hand-edit; every field is validated on load.

Migrations are applied automatically and non-destructively:

- **localStorage → `layout.json`** — the pre-v1 Zustand `persist` blob is read once, rewritten to disk, and only then removed from localStorage.
- **v1 → v2** — press effects renamed (`pulse`→`glow-pulse`, `ripple`→`border-ripple`, `bounce`→`key-drop`, `trail`→`glow`), themes renamed (`neumorphism`→`stealth-minimal`, `rgb-gradient`→`rgb-wave`), `snapToGrid` / `gridSize` added, and `soundEnabled` / `hotkeyToggleOverlay` dropped because neither had an implementation behind it.

---

## Architecture

```
src/                              React frontend
├── App.tsx                       composition root; gates render on hydration
├── components/
│   ├── OverlayCanvas.tsx         key rendering + all edit-mode pointer interaction
│   ├── KeyElement.tsx            one key; subscribes to its own press state only
│   ├── Toolbar.tsx               edit-mode tools, alignment, grid snap
│   ├── FloatingControlBar.tsx    hover-revealed window controls
│   ├── CustomizationDrawer.tsx   the single customization surface
│   ├── KpsMeter.tsx              isolated so the KPS tick re-renders nothing else
│   ├── drawer/                   Visuals, Themes, Animations, Layouts, Settings tabs
│   └── ui/controls.tsx           shared Toggle / Slider / ColorField / Segmented
├── hooks/
│   ├── useGlobalInput.ts         input-event and hotkey listeners, KPS tick
│   └── useProfilePersistence.ts  load on mount, debounced save on change
├── store/
│   ├── useOverlayStore.ts        Zustand: profile + ephemeral runtime state
│   └── persistence.ts            validation, migration, export / import
├── layouts/templates.ts          layout presets and visual themes
├── lib/color.ts                  CSS colour parsing for the pickers and glows
└── types/
    ├── input.ts                  the Rust wire contract
    └── overlay.ts                profile schema

src-tauri/src/
├── main.rs                       thin entry point; sets the Windows subsystem
├── lib.rs                        plugins, global shortcuts, tray, invoke handler
├── input_listener.rs             rdev thread, event translation, contract tests
└── commands.rs                   window control and profile file I/O
```

**Input path.** `rdev` runs on a detached thread and translates each event into a single `input-event` payload (`device`, `action`, `code`, `label`, `timestamp`), emitted to the `main` window. `code` uses W3C `KeyboardEvent.code` names for keys and `Mouseleft` / `Mouseright` / `Mousemiddle` for buttons. Four unit tests in `input_listener.rs` pin this contract; run them with `cargo test --lib`.

**Render path.** Press state lives in the store as an ephemeral `Set` of held codes plus a counter map, deliberately outside the persisted profile. Each `KeyElement` subscribes to only its own code, so a keystroke re-renders exactly one key component plus the isolated KPS meter — not the canvas, not the toolbar. Drag and resize keep their in-flight geometry in a ref and expose it as CSS custom properties on the canvas, so moving a whole multi-selection costs zero React renders and zero store writes; the store is written once, on pointerup.

---

## Known limitations

**`rdev` is X11-only on Linux.** Under a Wayland session, `rdev::listen()` returns no events and **global capture silently does nothing** — no error, no warning. XWayland does not help, because Wayland compositors deliberately do not route global input to X clients. Wayland support would need a different backend entirely (a compositor-specific protocol, or `libinput` with the user in the `input` group). Use an X11 session if you need this to work today.

**`stop_input_listener` does not stop the OS hook.** `rdev::listen()` blocks forever and has no cancellation. The command stops *forwarding* events to the UI; the hook itself lives until the process exits. This is a limitation of `rdev`, and the code says so rather than pretending otherwise.

**No sound.** Audio feedback on press is not implemented and the setting was removed rather than shipped as an inert toggle.

Not implemented, in rough order of how often they are missed:

- **Multiple saved profiles.** `ProfileConfig` carries an `id`, but only one file is ever written. Export/import is the workaround.
- **Rebindable hotkeys.** `Ctrl+Shift+O` and `Ctrl+Shift+L` are registered statically in `lib.rs`. Making them configurable needs dynamic re-registration on the Rust side.
- **Undo / redo.** No edit history.
- **Marquee (rubber-band) selection.** Multi-select is shift+click only.
- **Touch events.** `rdev` does not report them. Touchscreens synthesise mouse clicks, so a left-click zone doubles as a touch zone — which is why there is no separate "touch zone" control.
- **Key rotation.** `KeyConfig.rotation` is honoured by the renderer but no UI sets it.
