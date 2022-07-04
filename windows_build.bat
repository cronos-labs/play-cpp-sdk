git config --global core.symlinks true
git submodule update --init --recursive
cargo build --package play-cpp-sdk --release
cd demo
msbuild .\demo.sln -t:rebuild -property:Configuration=Release /p:Platform=x64
cmake.exe ^
    -B "out\build\x64-Release" ^
    -G "Ninja" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_INSTALL_PREFIX:PATH="out\install\x64-Release" ^
    -DCMAKE_C_COMPILER="cl.exe" ^
    -DCMAKE_CXX_COMPILER="cl.exe" ^
    -DCMAKE_MAKE_PROGRAM="ninja.exe" ^
    .
cd out\build\x64-Release
ninja
cd ..\..\..\..
