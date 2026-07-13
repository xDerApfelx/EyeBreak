# 👁️ EyeBreak

**Your eyes need breaks. You forget them. EyeBreak doesn't.**

EyeBreak is a tiny desktop app that reminds you to look away from your screen every hour — with an overlay that even shows up over your games. Built because manual timers get dismissed and forgotten, and eye doctors keep being right.

## Table of Contents

- [Download & Install 📥](#download--install-)
- [User Guide](#user-guide)
- [Core Features](#core-features)
- [How it Works](#how-it-works)
- [The Story behind EyeBreak 🚀](#the-story-behind-eyebreak-)
- [⚠️ Flaws & Limitations](#️-flaws--limitations)
- [Platform Support](#platform-support)
- [❓ FAQ](#-faq)
- [For Nerds 🤓](#for-nerds-)
- [📜 License](#-license)

* * *

## Download & Install 📥

Grab the latest release for your platform:

- 🪟 **Windows**: `EyeBreak_x.x.x_x64-setup.exe` (installer) or `.msi`
- 🐧 **Linux**: `.AppImage` (portable) or `.deb`
- 🍎 **macOS**: `.dmg`

Install it, start it once, done. EyeBreak lives in your system tray — there is no main window cluttering your taskbar.

## User Guide

1. **Start EyeBreak.** It sits in your tray (look for the icon with the green dot).
2. **Left-click the tray icon** to open settings: interval length, break length, difficulty, language (English/German), dark/light theme, autostart.
3. **Keep doing your thing.** After an hour of active use (default), the overlay appears in the top-right corner and counts down to your break.
4. **Take the break.** Look out the window, stare into the distance, pet your cat. After 5 minutes, the next hour starts automatically.

The overlay is **click-through** — it never steals your mouse from a game. Hover over it for a moment to reveal its buttons (postpone, start break early, skip).

### Difficulty levels

| | Postpone | Skip break | When you ignore it |
|---|---|---|---|
| 🟢 **Easy** | unlimited | instantly | nothing happens |
| 🟡 **Normal** | 2× | after 30 s | overlay stays on top |
| 🔴 **Strict** | never | never | your PC gets **locked** (nothing is closed — just log back in) |

## Core Features

- ⏱️ **Automatic hourly timer** — starts with your PC, no manual timers to forget
- 🖱️ **Idle detection** — away from the keyboard? The timer pauses. Away long enough to count as a break? The hour restarts. Fair.
- 🎮 **Game detection** — knows the games installed via Steam, Epic, GOG, Ubisoft Connect, Xbox and Battle.net. In a game, you get one extra postpone to finish the round.
- ⌨️ **Flow detection** — typing fast and steadily when the hour ends? The break politely waits (capped at +15 min, no cheating).
- 👻 **Click-through overlay** — visible over borderless-fullscreen games, invisible to your mouse until you hover on it
- 🚦 **Tray status light** — green: running, yellow: update available, red: something broke
- 🌍 **English & German**, dark & light theme, everything configurable

## How it Works

EyeBreak runs a state machine with three states: **Active** (counting down your hour), **Idle-Paused** (you left — timer frozen), and **Break** (countdown until your eyes are allowed back).

- Input activity is read from the OS (`GetLastInputInfo` on Windows) — EyeBreak never logs *what* you type, only *that* you typed.
- Game detection reads the launchers' own manifest files (the same technique [DLSS Swapper](https://github.com/beeradmoore/dlss-swapper) uses) — no game database to maintain.
- The overlay is a transparent always-on-top window that ignores mouse events until you deliberately hover it.

## The Story behind EyeBreak 🚀

Two friends, too many 5-hour gaming sessions, one eye doctor's advice: *look away for a few minutes every hour.* Phone timers? Dismissed and forgotten. So we did the only reasonable thing programmers do — spent hours building a tool to save minutes. This is that tool, and honestly, our eyes feel better already.

## ⚠️ Flaws & Limitations

Honesty is important:

- **Exclusive-fullscreen games** (mostly old DirectX 9 titles) render below the overlay. That would need DLL injection like Discord's overlay — we deliberately don't do that. Borderless fullscreen (the default in modern games) works fine.
- **Strict mode locks your session** — it never closes anything, but if that scares you, stay on Normal.
- **Flow detection counts keystrokes only** — a fast typing session extends your hour, mouse-only work doesn't.
- **The overlay is on your primary monitor only** (for now).
- Expect bugs. It's young software.

## Platform Support

| | 🪟 Windows | 🍎 macOS | 🐧 Linux |
|---|---|---|---|
| Timer + Overlay | ✅ | ✅ (not over fullscreen spaces) | ✅ |
| Idle detection | ✅ | ✅ | ✅ X11 (`xprintidle`) / ❌ Wayland |
| Flow detection | ✅ | ❌ | ❌ |
| Game detection | ✅ all 6 launchers | fullscreen heuristic only | fullscreen heuristic only |
| Lock (Strict) | ✅ | ❌ (no public API) | ✅ (`loginctl`) |

Windows is the primary platform. macOS and Linux are best-effort — the core timer works everywhere, the extras degrade gracefully.

## ❓ FAQ

**Does EyeBreak track what I type?**
No. It reads *whether* input happened (for idle detection) and *how many* keys per minute (for flow detection). No content, no logging, no network — your data never leaves your machine.

**Why does it want to start with my PC?**
Because a break reminder you have to remember to start is a paradox. It's optional — toggle it in settings.

**Can I use it for the 20-20-20 rule instead?**
Sure — set the interval to 20 minutes and the break to 1 minute.

**The overlay doesn't show over my game!**
Your game probably runs in exclusive fullscreen. Switch it to *borderless fullscreen* — you get faster alt-tabbing as a bonus.

**Something broke!**
Open an issue. Include your OS, what you did, and what happened instead of the expected thing.

## For Nerds 🤓

Built with [Tauri 2](https://tauri.app) — Rust backend, vanilla HTML/CSS/JS frontend, ~30 MB RAM, no Electron.

```sh
npm install
npm run tauri dev     # dev build with hot reload
npm run tauri build   # release build + installers
```

Prerequisites: [Rust](https://rustup.rs/), Node.js, and on Windows the VS Build Tools (C++ workload). For quick testing, set the interval to 1–2 minutes in the settings.

Releases are built automatically by GitHub Actions for all three platforms on every `v*` tag.

<details>
<summary>Enabling update notifications & auto-update (maintainers)</summary>

- Update notification (yellow tray dot): set `UPDATE_REPO` in `src-tauri/src/updater.rs` to `Some("owner/repo")`.
- Full silent auto-update (Windows/Linux): generate a signing key pair (`npm run tauri signer generate`), add the public key and endpoint to `tauri.conf.json` (`plugins.updater`), set `bundle.createUpdaterArtifacts: true`, wire up `tauri-plugin-updater`, and store the private key as the `TAURI_SIGNING_PRIVATE_KEY` GitHub secret.
- macOS auto-update additionally requires Apple notarization (99 $/year) — deliberately postponed.

</details>

## 📜 License

MIT — do whatever, just don't blame us for your eyesight.
