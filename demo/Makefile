UNAME := $(shell uname)

ifeq ($(UNAME), Darwin)
	FLAGS=-framework Security -framework CoreFoundation
endif

ifeq ($(UNAME), Linux)
	FLAGS=-lpthread -lssl -lcrypto -ldl
endif

all: clone build
build: prepare static dynamic
run: run_static run_dynamic

clone:
	git submodule update --init --recursive

prepare:
	python3 helper.py --target_dir ../target/release

prepare_x86_64:
	python3 helper.py --target_dir ../target/x86_64-apple-darwin/release

easywsclient.o:
	g++ -c third_party/easywsclient/easywsclient.cpp

easywsclient.o.x86_64:
	arch -x86_64 g++ -c third_party/easywsclient/easywsclient.cpp

static: easywsclient.o
	g++ -o demostatic \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		lib/libplay_cpp_sdk.a \
		-std=c++14 $(FLAGS)

dynamic: easywsclient.o
	g++ -o demo \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		include/*.cc \
		include/extra-cpp-bindings/src/*.cc \
		include/defi-wallet-core-cpp/src/*.cc \
		lib/libcxxbridge1.a \
		-lplay_cpp_sdk \
		-std=c++14 \
		$(FLAGS) \
		-L lib \

# test whether dylib loading correctly
x86_64_test_dylib:
	./demo test

x86_64_build: x86_64_build_static x86_64_build_dynamic

x86_64_build_test:x86_64_build_dynamic x86_64_test_dylib
	
x86_64_build_static: prepare_x86_64 easywsclient.o.x86_64
	arch -x86_64 \
		g++ -o demostatic \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		lib/libplay_cpp_sdk.a \
		-std=c++14 \
		$(FLAGS)

# use -rpath option to add the path to the dynamic library loading path
# -rpath lib in gcc or
# install_name_tool -add_rpath @executable_path/lib ./demo
# in mac, to use DYLD_LIBRARY_PATH, it is normalized to use $rpath in dylib
x86_64_build_dynamic : prepare_x86_64 easywsclient.o.x86_64
	install_name_tool -id @rpath/libplay_cpp_sdk.dylib ./lib/libplay_cpp_sdk.dylib
	otool -L ./lib/libplay_cpp_sdk.dylib
	arch -x86_64 \
		g++ -o demo \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		./include/walletconnectcallback.cc \
		./include/nft.cc \
		./include/pay.cc \
		./include/defi-wallet-core-cpp/src/contract.rs.cc  \
		./include/defi-wallet-core-cpp/src/core.cc \
		./include/defi-wallet-core-cpp/src/nft.rs.cc \
		./include/defi-wallet-core-cpp/src/uint.rs.cc \
		./include/extra-cpp-bindings/src/lib.rs.cc  \
		lib/libplay_cpp_sdk.dylib \
		lib/libcxxbridge1.a \
		-std=c++14 \
		-rpath lib \
		$(FLAGS)
	

run_static:
	. ./.env && ./demostatic

run_dynamic:
	. ./.env && export LD_LIBRARY_PATH=$(PWD)/lib && ./demo

clean:
	rm easywsclient.o demo demostatic
