#!/bin/bash

if [ -z "$1" ]; then
  arch="$(uname -m)"
else
  arch="$1"
fi

root_dir="$(dirname "$(cargo locate-project --workspace --message-format plain)")"
pushd "$root_dir" > /dev/null

version=$(grep -E '^version\s*=\s*".*"' Cargo.toml | sed -E 's/version\s*=\s*"(.*)"/\1/')
distro=$(kreadconfig6 --file /etc/os-release --group "<default>" --key "ID")

cargo build --release --target "${arch}-unknown-linux-gnu"

mkdir package
cp -r frontend/kde/* package/
sed -i "s/LYRICA_VERSION/${version}/g" package/metadata.json
mkdir -p package/contents/bin/
cp "target/${arch}-unknown-linux-gnu/release/lyrica" package/contents/bin/
7z a -tzip "lyrica-plasmoid-v${version}-${distro}-${arch}.plasmoid" package/*
rm -rf package

popd > /dev/null
