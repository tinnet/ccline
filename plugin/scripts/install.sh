#!/bin/sh
# Downloads the prebuilt ccline binary matching this plugin's version into
# ${CLAUDE_PLUGIN_DATA}/bin, which survives plugin updates, so the statusLine
# command in settings.json can point at a stable path.
#
# Runs on every SessionStart; a no-op when the installed version already matches.
# Prints the binary path on success (unless --quiet).
set -eu

QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

log() { [ "$QUIET" = 1 ] || echo "ccline: $*" >&2; }
fail() {
    echo "ccline: $*" >&2
    # Never break session start over a failed download; the old binary (if any) stays.
    [ "$QUIET" = 1 ] && exit 0
    exit 1
}

ROOT="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
DATA="${CLAUDE_PLUGIN_DATA:-}"
[ -n "$DATA" ] || fail "CLAUDE_PLUGIN_DATA is not set"

VERSION=$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' "$ROOT/.claude-plugin/plugin.json" | head -n 1)
[ -n "$VERSION" ] || fail "could not read version from plugin.json"

case "$(uname -s)" in
    Darwin) OS=apple-darwin ;;
    Linux) OS=unknown-linux-gnu ;;
    MINGW* | MSYS* | CYGWIN*) OS=pc-windows-msvc ;;
    *) fail "unsupported OS: $(uname -s)" ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) ARCH=x86_64 ;;
    arm64 | aarch64) ARCH=aarch64 ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
esac
TARGET="$ARCH-$OS"

BIN_NAME=ccline
EXT=tar.xz
if [ "$OS" = pc-windows-msvc ]; then
    BIN_NAME=ccline.exe
    EXT=zip
    [ "$ARCH" = x86_64 ] || fail "no prebuilt binary for $TARGET"
fi

BIN_DIR="$DATA/bin"
BIN="$BIN_DIR/$BIN_NAME"
STAMP="$BIN_DIR/.version"

if [ -x "$BIN" ] && [ "$(cat "$STAMP" 2>/dev/null)" = "$VERSION" ]; then
    [ "$QUIET" = 1 ] || echo "$BIN"
    exit 0
fi

ARCHIVE="ccline-$TARGET.$EXT"
URL="https://github.com/tinnet/ccline/releases/download/v$VERSION/$ARCHIVE"

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 2 -o "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        return 1
    fi
}

mkdir -p "$BIN_DIR"
TMP=$(mktemp -d "$DATA/download.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

log "downloading $URL"
download "$URL" "$TMP/$ARCHIVE" || fail "download failed: $URL"

# Verify the checksum when a sha256 tool is available.
if download "$URL.sha256" "$TMP/sum" 2>/dev/null; then
    EXPECTED=$(cut -d ' ' -f 1 <"$TMP/sum")
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "$TMP/$ARCHIVE" | cut -d ' ' -f 1)
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL=$(shasum -a 256 "$TMP/$ARCHIVE" | cut -d ' ' -f 1)
    else
        ACTUAL=$EXPECTED
    fi
    [ "$ACTUAL" = "$EXPECTED" ] || fail "checksum mismatch for $ARCHIVE"
fi

if [ "$EXT" = zip ] && command -v unzip >/dev/null 2>&1; then
    unzip -q "$TMP/$ARCHIVE" -d "$TMP" || fail "could not extract $ARCHIVE"
else
    tar -xf "$TMP/$ARCHIVE" -C "$TMP" || fail "could not extract $ARCHIVE"
fi

# Archives contain ccline-<target>/<bin> (cargo-dist layout); tolerate a flat layout too.
SRC="$TMP/ccline-$TARGET/$BIN_NAME"
[ -f "$SRC" ] || SRC="$TMP/$BIN_NAME"
[ -f "$SRC" ] || fail "$BIN_NAME not found in $ARCHIVE"

chmod +x "$SRC"
mv -f "$SRC" "$BIN.new"
mv -f "$BIN.new" "$BIN"
echo "$VERSION" >"$STAMP"

log "installed v$VERSION"
[ "$QUIET" = 1 ] || echo "$BIN"
