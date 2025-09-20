#!/bin/sh

SCRIPT_DIR=$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)

SRC_DIR="$SCRIPT_DIR/fgmapeditor/items"
DST_DIR="$SCRIPT_DIR/fgserver/items"

for file in "$SRC_DIR"/*.tres; do
  if [ -e "$file" ]; then
    cp "$file" "$DST_DIR/" || {
      echo "Failed to copy $file" >&2
      exit 1
    }
  fi
done
