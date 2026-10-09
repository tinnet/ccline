# Changelog

Notable changes to ccline. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.8.0] - 2026-10-09

### Added
- Once a rate limit window reaches 80%, it also shows the time until it resets (`5h 85% ↻1h20m`).

## [0.7.0] - 2026-10-09

### Added
- Subscription rate limit usage (`5h 23% 7d 41%`), colored green, yellow or red as it fills. Shown only when Claude Code sends it (Pro/Max).

### Changed
- The cost reads `~$0.12`: what the session would cost at API list price, not what a subscription pays.
- Whole-million context windows read `1M`, not `1.0M`.
- `bench/bench-input.json` now contains every field from the current status line docs.

## [0.6.0] - 2026-10-09

### Added
- Per-repo mark before the path: one of 12 shapes × 6 colors, hashed from the repo folder name, so sessions in different repos are easy to tell apart.

### Fixed
- Branch and dirty state now show when the current directory is a subdirectory of the repo.

### Changed
- libgit2 is compiled in, without HTTPS/SSH support. The binary no longer depends on system libgit2 or OpenSSL, and a run drops from ~14ms to ~4ms.
- Benchmarks run against the ccline checkout, so they include the git work.

## [0.5.0] - 2026-10-08

### Added
- Installable as a Claude Code plugin: the plugin downloads the release binary, and `/ccline:setup` configures the status line.

## [0.4.0] - 2026-09-24

### Changed
- Show the session cost only; the token count is gone from the status line.

## [0.3.0] - 2026-05-20

### Added
- Effort level next to the model name.
- Context window usage as a percentage of the window size.

### Changed
- Benchmarks compare against a POSIX shell equivalent under bash, sh and zsh, plus other status line tools.

## [0.2.0] - 2026-03-06

### Changed
- New layout: model, short path, branch, tokens and cost in muted Monokai Pro colors, replacing `user@host` and the full path.
- Token segment shows the context window size (`42k/200k tks`).

## [0.1.0] - 2026-03-06

### Added
- First release: reads Claude Code's status JSON on stdin and prints `user@host`, the current directory and the git branch with a dirty marker.
- Release builds for macOS, Linux and Windows via cargo-dist.

[Unreleased]: https://github.com/tinnet/ccline/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/tinnet/ccline/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/tinnet/ccline/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/tinnet/ccline/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/tinnet/ccline/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/tinnet/ccline/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tinnet/ccline/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/tinnet/ccline/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tinnet/ccline/releases/tag/v0.1.0
