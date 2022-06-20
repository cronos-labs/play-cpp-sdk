#!/usr/bin/env bash
PLATFORM=$(uname -s)

mkdir -p build
cp -r demo/include build
cp -r demo/lib build
cp ./LICENSE build
cp ./CHANGELOG.md build
cd build
tar zcvf ../play_cpp_sdk_${PLATFORM}_x86_64.tar.gz *
cd ..
shasum -a 256 *.tar.gz > "checksums-$PLATFORM.txt"
