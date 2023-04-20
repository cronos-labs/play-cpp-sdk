# use rust like this
# pacman -S --needed mingw-w64-x86_64-toolchain mingw-w64-x86_64-rust
# msvc rust is not compatible with msys2 

# bring all source
git config --global core.symlinks true
git submodule update --init --recursive

# fix for msys2
sed -i 's/typedef __int8 int8_t;/\/\/ typedef __int8 int8_t;/' ./demo/third_party/easywsclient/easywsclient.cpp

# prepare
cargo build --release

# copy library
cd ./demo
python ./helper.py
cd ..

# compile
mkdir -p ./demo/build
cd ./demo/build
cmake -G "MSYS Makefiles" ..
make
cd ../..
echo "OK"

