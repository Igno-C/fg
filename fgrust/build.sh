#!/bin/sh

# Currently needed because building in workspace mode (just a cargo build) leaves all features enabled
# leaving extra and unnecessary features in the resulting binaries
# Fix for this is in the works: https://github.com/rust-lang/cargo/issues/14774
cargo build -p rgdext "$@"
cargo build -p rgdext_client "$@"
cargo build -p rgdext_serverutil "$@"
cargo build -p map_benchmark "$@"
