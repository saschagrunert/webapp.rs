#!/usr/bin/env nix-shell
#!nix-shell -i /bin/sh -p coreutils gnused

sed -n -e 's/^const TAG: &str = "\(.*\)";$/\1/p' ../../frontend/build.rs | tr -d '\n'
