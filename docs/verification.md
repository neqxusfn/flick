# Verification — 12 October 2026

## Passed locally

- Windows x64 Tauri development build and native window launch.
- Optimized x64 standalone executable and NSIS installer build (about 3.5 MiB and 1.4 MiB respectively).
- 10 Rust unit tests: mode conversions, invalid values, fractional drift over 10,000 turns, exact step sums in both directions, JSON round trip, backward-compatible account-risk acknowledgment, invalid profiles, atomic replacement, repeat suppression, paired releases after rebinding, capture and full-channel handling.
- Clippy with `-D warnings`, `cargo fmt`, TypeScript production build, ESLint and Prettier checks.
- 2 frontend count conversion tests.
- 12 browser interaction/scaling checks: light/dark × 100%/125%/150%; first-use risk notice and re-prompt on attempted enable, count changes, multiple calibration modes, invalid input, profile creation/deletion, Settings Escape handling, viewport bounds and honest native-only preview errors.
- Visual inspection of screenshots at all three device scales in both themes.
- Native Windows visual inspection in dark and light, enable toggle, Test flick action without an input error, Settings navigation, and appearance saved to `%APPDATA%/Flick/profiles.json`.
- Standalone release launch with the development server stopped; saved appearance loaded correctly. Launching a second copy exited the duplicate and retained one responding release process.

## Verified on GitHub

- The `v0.1.0` Windows build and release jobs passed, including formatting, lint, frontend and browser tests, Rust lint and tests, and an optimized NSIS build ([workflow run](https://github.com/neqxusfn/flick/actions/runs/38148181493)).
- The [published release](https://github.com/neqxusfn/flick/releases/tag/v0.1.0) contains the installer and portable executable with SHA-256 digests.

## Still requires hands-on checks

- Physical XButton/middle/keyboard triggers and their effects in specific games; injected automation events deliberately cannot stand in for physical trigger testing.
- In-game 180° accuracy, Raw Input calibration against landmarks, return-on-release timing and compatibility with each game's input/anti-cheat policy.
- Real Windows 125%/150% display settings and mixed-DPI monitors (the automated scaling checks use browser device scale).
- Startup after Windows sign-in, UAC relaunch acceptance/cancellation with an elevated game, and a full installer/uninstaller run on a clean machine.

Preset research is recorded separately in [yaw-sources.md](yaw-sources.md). No unverified game constants were filled in.
