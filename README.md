# 👁️ AugenSchonen

Erinnert dich nach jeder Stunde aktiver PC-Nutzung an eine 5-Minuten-Augenpause — automatisch, unübersehbar, auch über Spielen.

## Features

- **Automatischer Timer**: startet mit dem PC, läuft im Tray, kein manuelles Stellen
- **Idle-Erkennung**: bist du länger weg (Standard: 5 Min.), pausiert der Timer; war die Abwesenheit so lang wie eine Pause, beginnt die Stunde neu
- **Overlay**: erscheint in den letzten 10 Minuten oben rechts, auch über Borderless-Fullscreen-Spielen; klick-durchlässig (stört das Spiel nicht), wird per Hover interaktiv
- **Animationen**: sichtbares Auftauchen bei 10 Min. → Blinken bei 5 Min. → Dauerblinken ab 1:30
- **Schwierigkeitsgrade**:
  - *Locker* — sofort wegklickbar, unbegrenzt verlängerbar
  - *Normal* — Überspringen erst nach 30 s, max. 2× verlängern
  - *Streng* — kein Verlängern, Vollbild-Block + PC wird gesperrt (nichts geht verloren, nur neu anmelden)
- **Spiel-Erkennung**: erkennt installierte Spiele von Steam, Epic, GOG, Ubisoft Connect, Xbox und Battle.net (Technik nach Vorbild von [DLSS Swapper](https://github.com/beeradmoore/dlss-swapper)); im Spiel gibt es einen Extra-Aufschub, um die Runde zu Ende zu spielen
- **Flow-Erkennung**: wer beim Ablauf gerade schnell und anhaltend tippt, bekommt die Pause minutenweise aufgeschoben — gedeckelt auf max. +15 Min./Intervall
- **Tray-Ampel**: grün = läuft, gelb = Update verfügbar, rot = Fehler
- **Einstellbar**: Intervall-, Pausen- und Puffer-Länge, Dark/Light, Autostart

## Entwicklung

Voraussetzungen: [Rust](https://rustup.rs/), Node.js, unter Windows die VS Build Tools (C++-Workload).

```sh
npm install
npm run tauri dev     # Dev-Build mit Hot-Reload
npm run tauri build   # Release-Build + Installer
```

Zum schnellen Testen in den Einstellungen einfach Intervall auf 1–2 Minuten stellen.

## Plattform-Support

| | Windows | macOS | Linux |
|---|---|---|---|
| Timer + Overlay | ✅ | ✅ (nicht über Fullscreen-Spaces) | ✅ |
| Idle-Erkennung | ✅ | ✅ (via `ioreg`) | ✅ X11 (`xprintidle` nötig) / ❌ Wayland |
| Flow-Erkennung | ✅ | ❌ (Tier 2) | ❌ (Tier 2) |
| Spiel-Erkennung | ✅ alle 6 Launcher | nur Vollbild-Heuristik | nur Vollbild-Heuristik |
| Sperren (Streng) | ✅ | ❌ (keine öffentliche API) | ✅ (`loginctl`) |

Windows ist Tier 1 (voller Umfang), macOS/Linux sind Best-Effort. Ein Overlay über *exclusive* Fullscreen (alte DirectX9-Spiele) ist prinzipbedingt nicht möglich — Borderless/Windowed funktioniert.

## Releases & Updates

GitHub Actions baut bei jedem `v*`-Tag automatisch Windows- (.msi/.exe), Linux- (.AppImage/.deb) und macOS-Artefakte (.dmg) als Release-Draft.

Update-Benachrichtigung (gelber Tray-Punkt): in [src-tauri/src/updater.rs](src-tauri/src/updater.rs) `UPDATE_REPO` auf `Some("owner/repo")` setzen, sobald das GitHub-Repo existiert.

**TODO — volles Silent-Auto-Update** (Windows/Linux): sobald das Repo öffentlich ist,
1. `npm run tauri signer generate` — Schlüsselpaar erzeugen
2. Public Key + Endpoint in `tauri.conf.json` (`plugins.updater`) eintragen, `bundle.createUpdaterArtifacts: true`
3. `tauri-plugin-updater` einbinden, privaten Schlüssel als GitHub-Secret (`TAURI_SIGNING_PRIVATE_KEY`) hinterlegen

macOS-Auto-Update braucht zusätzlich Apple-Notarization (99 $/Jahr) — bewusst aufgeschoben.
