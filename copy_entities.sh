#!/bin/sh

SCRIPT_DIR=$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)

SRC_DIR="$SCRIPT_DIR/fgmapeditor/entities"
DST_DIR="$SCRIPT_DIR/fgserver/entities"

for file in "$SRC_DIR"/*.gd; do
  if [ -e "$file" ]; then
    cp "$file" "$DST_DIR/" || {
      echo "Failed to copy $file" >&2
      exit 1
    }
  fi
done
