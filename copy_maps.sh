#!/bin/sh

SCRIPT_DIR=$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)

SRC_DIR1="$SCRIPT_DIR/fgmapeditor/exports/client"
DST_DIR1="$SCRIPT_DIR/fgclient/maps"

SRC_DIR2="$SCRIPT_DIR/fgmapeditor/exports/server"
DST_DIR2="$SCRIPT_DIR/fgserver/maps"

cp -R "$SRC_DIR1"/. "$DST_DIR1"/ || {
  echo "cp1 failed" >&2
  exit 1
}

cp -R "$SRC_DIR2"/. "$DST_DIR2"/ || {
  echo "cp2 failed" >&2
  exit 1
}
