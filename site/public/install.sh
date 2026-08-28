#!/bin/sh
set -eu
REPO="B-Divyesh/sf-pdf-redaction-proof"
BASE="https://github.com/$REPO/releases/latest/download"
OS=$(uname -s)
ARCH=$(uname -m)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

case "$OS:$ARCH" in
  Darwin:arm64) PATTERN='universal.dmg' ;;
  Darwin:x86_64) PATTERN='universal.dmg' ;;
  Linux:x86_64) PATTERN='amd64.AppImage' ;;
  Linux:aarch64) PATTERN='aarch64.AppImage' ;;
  *) echo "Unsupported platform: $OS $ARCH" >&2; exit 1 ;;
esac

curl -fsSL "$BASE/SHA256SUMS" -o "$TMP_DIR/SHA256SUMS"
ASSET=$(awk -v pattern="$PATTERN" '$2 ~ pattern {print $2; exit}' "$TMP_DIR/SHA256SUMS")
if [ -z "$ASSET" ]; then echo "No matching release asset for $OS $ARCH." >&2; exit 1; fi
curl -fL "$BASE/$ASSET" -o "$TMP_DIR/$ASSET"
(cd "$TMP_DIR" && grep "  $ASSET$" SHA256SUMS | sha256sum -c -)

if [ "$OS" = "Linux" ]; then
  mkdir -p "$HOME/.local/bin"
  install -m 755 "$TMP_DIR/$ASSET" "$HOME/.local/bin/redaction-proof"
  echo "Installed Redaction Proof to $HOME/.local/bin/redaction-proof (SHA-256 verified)."
  echo "Add $HOME/.local/bin to PATH if it is not already there."
else
  DEST="$HOME/Downloads/$ASSET"
  cp "$TMP_DIR/$ASSET" "$DEST"
  echo "Downloaded and verified $DEST. Open it, then right-click Redaction Proof and choose Open (the v1 build is unsigned)."
fi
