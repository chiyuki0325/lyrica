#!/bin/bash

if [ -z "$1" ]; then
  arch="$(uname -m)"
else
  arch="$1"
fi

version=$(jq -cr ".KPlugin.Version" plasmoid/metadata.json)
distro=$(kreadconfig6 --file /etc/os-release --group "<default>" --key "ID")

cargo build --release --target "${arch}-unknown-linux-gnu"

mkdir package
cp -r plasmoid/* package/
mkdir -p package/contents/bin/
cp "target/${arch}-unknown-linux-gnu/release/lyrica" package/contents/bin/
7z a -tzip "lyrica-plasmoid-v${version}-${distro}-${arch}.plasmoid" package/*
rm -rf package
