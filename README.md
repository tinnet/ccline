# ccline

A fast status line for [Claude Code](https://docs.anthropic.com/en/docs/claude-code), written in Rust.

Reads workspace JSON from stdin, outputs an ANSI-formatted status line showing user, host, working directory, git branch, and dirty state.

```
user@hostname ~/projects/myapp main*
```

## Why

Claude Code lets you set a custom `statusLine` command that runs on every prompt refresh. The default approach is a bash script, but spawning bash + jq + git on every invocation is slow (~280ms). This Rust binary does the same thing in ~1.4ms — about 200x faster.

## Install

```bash
cargo install --path .
```

## Usage

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "ccline"
  }
}
```

Claude Code pipes JSON to stdin on each refresh:

```json
{"workspace": {"current_dir": "/path/to/project", "project_dir": "/path/to/project", "added_dirs": []}}
```

## Layout

Hardcoded, no config. Fork and edit `src/main.rs` to customize.

| Segment | Color | Source |
|---------|-------|--------|
| `user@host` | default | `$USER` + `gethostname` |
| `/path` | blue | `workspace.current_dir` |
| `branch` | gray | `git2` |
| `*` | cyan | dirty working tree |

## License

MIT
