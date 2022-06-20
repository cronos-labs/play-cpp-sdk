#!/usr/bin/env bash
PLATFORM=$(uname -s)

mkdir -p build
cp -r demo/include build
cp -r demo/lib build
cp ./LICENSE build
cp ./CHANGELOG.md build
