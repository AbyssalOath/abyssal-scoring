# Abyssal Live (iPad app)

A SwiftUI app for judges (score gymnastics/swimming from an iPad) and
spectators (watch the live scoreboard) - the native client for the
`live_server` API in the main Cargo workspace. See
[`../docs/LIVE_API.md`](../docs/LIVE_API.md) for the full wire
contract this app is built against, and
[`../ARCHITECTURE.md`](../ARCHITECTURE.md)'s "Networked live judging"
section for the backend design.

**Important - read before opening this in Xcode:** this project was
written and scaffolded on a Linux machine with no access to Xcode, a
Swift compiler, or any Apple toolchain. Every file has been reviewed
by hand and checked for obvious structural issues, but **none of it
has actually been compiled or run.** Treat this as a solid starting
point to open, build, and fix up in Xcode - not as verified, working
code the way the Rust side of this repo is (which has real automated
test coverage; see `crates/live_server/tests/api.rs`).

## Prerequisites

- A Mac with Xcode 15+ installed.
- [XcodeGen](https://github.com/yonaskolb/XcodeGen): `brew install xcodegen`.
  This project's `.xcodeproj` is generated from `project.yml`, not
  committed - the usual way to avoid painful pbxproj merge conflicts.
- A free Apple ID is enough to build and run on your own device via
  Xcode. TestFlight distribution to other judges' iPads needs a paid
  Apple Developer Program membership ($99/year).

## Building

```bash
cd ios
xcodegen generate
open AbyssalLive.xcodeproj
```

In Xcode: select the `AbyssalLive` target, go to **Signing &
Capabilities**, and set your own Team (Automatic signing is already
configured in `project.yml`). Then build and run on the Simulator or
a real iPad.

Re-run `xcodegen generate` any time you add/remove/rename a Swift
file - it re-scans `AbyssalLive/` and regenerates the project.

## Trying it against the real backend

You don't need the full Tauri desktop app running to test this - the
backend crate has a standalone example server for exactly this:

```bash
# From the repo root, on whatever machine will run the API:
cargo run -p live_server --example smoke_server
```

This prints a pairing code and seeds one competition/judge/event/
participant into an in-memory database, then serves on port 7878. In
the app:
- **I'm Judging** → enter that machine's IP address (`localhost` if
  you're running in the Simulator on the same Mac; a real LAN IP if
  testing from a physical iPad) and the printed pairing code.
- **I'm Watching** → enter the same address, no code needed - pick the
  seeded competition from the list.

To test against the real desktop app instead, use its "Live Judging"
tab to see the server's LAN address and generate a real pairing code
for a judge you've added.

## What's here

```
AbyssalLive/
├── AbyssalLiveApp.swift       App entry point
├── RootView.swift             Judge vs. Spectator mode picker
├── Models/                    Codable types mirroring the Rust wire
│                               format exactly - see LIVE_API.md
├── Networking/
│   ├── APIClient.swift        REST calls (URLSession, async/await)
│   └── ScoreboardSocket.swift WebSocket client with auto-reconnect
├── Session/
│   ├── ServerSession.swift    Connection state (host/port/token/judge),
│   │                           persisted across launches
│   └── KeychainStore.swift    Device token storage (Keychain, not
│                               UserDefaults - it's the auth credential)
├── Judge/                     Pairing → event/participant picker →
│                               gymnastics or swimming scoring form
└── Spectator/                 Server/competition picker → live
                                scoreboard view
```

## Known gaps / good next steps

- **No QR-code pairing** - typing a 6-digit code + IP address works
  but is more friction than scanning a code the desktop could display.
  Would need `NSCameraUsageDescription` added to `project.yml` and a
  scanner (e.g. `AVCaptureSession` or the `CodeScanner` package).
- **Sport is a manual picker in `JudgeHomeView`**, not auto-detected -
  `/pair`'s response doesn't currently carry the competition's sport.
  Threading `Competition.sport` through that response (or having the
  app call `GET /competitions` and cross-reference) would remove this
  manual step.
- **No unit tests** on the Swift side yet (`APIClient`/`ScoreboardSocket`
  are structured to be testable with a mock `URLProtocol`, but none
  are written). The Rust side's contract is tested
  (`crates/live_server/tests/api.rs`); this app trusts that contract
  without its own automated coverage.
- **Placeholder app icon/accent color** (`Assets.xcassets`) - fine for
  Simulator/local device builds, needs a real icon before TestFlight.
- Everything is LAN-only, matching the backend - there's no "watch
  from home" mode, and building one is a separate, larger decision
  (see ARCHITECTURE.md).
