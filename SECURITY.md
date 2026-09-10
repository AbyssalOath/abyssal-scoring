# Security Policy

## Supported versions

Only the latest tagged release receives security fixes - there's no
long-term-support branch plan at this stage.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting rather than
opening a public issue: go to the **Security** tab on this repo →
**Report a vulnerability**. That opens a private draft advisory only
visible to maintainers until a fix is ready.

If you're not able to use that flow for some reason, opening a
regular issue with as little exploit detail as possible (just "I
think there's a security issue in X, can we talk privately") is a
reasonable fallback - a maintainer will reach out for details.

Please allow a reasonable window to investigate and patch before any
public disclosure.

## Scope and what to expect

Abyssal Scoring is a **local-first desktop application**. Some
context that shapes what "security" means here:

- **No network service.** The app doesn't run a listening server, a
  web API, or accept incoming network connections. All
  frontend-to-backend communication is local Tauri IPC between the
  app's own windows and its own bundled Rust binary - there's no
  remote attack surface by default.
- **No user accounts, no authentication.** Whoever has access to the
  computer the app is installed on has access to its data. This is
  intentional (it's meant to run on a scorer's own laptop at an
  event), but it does mean the app is not designed to protect
  competition data from someone with local access to the machine.
- **Data at rest is unencrypted.** The SQLite database
  (`abyssal.sqlite`, in the OS's standard app-data directory) and any
  exported `.abyssal` files are plain, unencrypted SQLite/zip+JSON.
  `.abyssal` exports contain participant, judge, and team names -
  treat exported files with the same care you'd give any file
  containing people's names, and don't assume the `.abyssal` format
  itself provides any confidentiality.
- **Import trust.** Importing an `.abyssal` file writes its contents
  (including any referenced logo/background images) directly into
  the local database and app-data asset directory. Only import files
  from sources you trust - the importer does not sandbox or deeply
  validate image contents beyond copying the bytes.
- **Third-party dependencies.** This project depends on `tauri`,
  `sqlx`, `zip`, and their transitive dependency trees. Vulnerabilities
  in those are best reported upstream, though flagging them here is
  also fine if you're not sure where a given issue actually lives.

Genuinely find something concerning here - a way to escalate from
"reads/writes local app data" to something broader (e.g. arbitrary
code execution via a crafted `.abyssal` file or image asset, a path
traversal in the export/import asset handling, etc.) - that's exactly
the kind of thing worth a private report.
