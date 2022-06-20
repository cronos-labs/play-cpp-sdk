git config --global core.symlinks true
git submodule update --init --recursive
cargo build --package play-cpp-sdk --release
cd demo
msbuild .\demo.vcxproj -t:rebuild -property:Configuration=Release /p:Platform=x64
msbuild .\demostatic.vcxproj -t:rebuild -property:Configuration=Release /p:Platform=x64
cd ..
