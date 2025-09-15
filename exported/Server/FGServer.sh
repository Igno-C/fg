#!/bin/sh
echo -ne '\033c\033]0;FGServer\a'
base_path="$(dirname "$(realpath "$0")")"
"$base_path/FGServer.x86_64" "$@"
