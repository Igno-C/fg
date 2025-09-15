#!/bin/sh
echo -ne '\033c\033]0;FGAuth\a'
base_path="$(dirname "$(realpath "$0")")"
"$base_path/FGAuth.x86_64" "$@"
