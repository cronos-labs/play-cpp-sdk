#!/usr/bin/env bash
PLATFORM=$(uname -s)

mkdir -p install
cp -r demo/sdk install
cp ./LICENSE install/sdk
cp ./CHANGELOG.md install/sdk
