#!/bin/bash

if [ -z "$1" ]; then
  arch="$(uname -m)"
else
  arch="$1"
fi

cargo build --release --target "${arch}-unknown-linux-gnu"

mkdir package
cp -r plasmoid/* package/
cp "target/${arch}-unknown-linux-gnu/release/lyrica" package/contents/bin/
7z a -tzip "lyrica-plasmoid-${arch}.plasmoid" package/*
rm -rf package
