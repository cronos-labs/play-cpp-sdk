# Compile with g++: make CXX=g++
# Compile with clang++: make CXX=clang++
# Compile with default compiler: make
UNAME := $(shell uname)
sdk_dir = ./sdk

GCC_CXXFLAGS =
CLANG_CXXFLAGS = -stdlib=libc++
DEFAULT_CXXFLAGS =

ifeq ($(CXX),g++)
  CXXFLAGS += $(GCC_CXXFLAGS)
else ifneq (,$(findstring clang,$(CXX)))
  CXXFLAGS += $(CLANG_CXXFLAGS)
else
  CXXFLAGS += $(DEFAULT_CXXFLAGS)
endif

ifeq ($(shell uname -m), x86_64)
 ifeq ($(UNAME), Darwin)
  ifeq ($(USE_ARM64),true) 
  CXX = clang++ -target arm64-apple-darwin # for arm64, we only use clang++
  TARGET_DIR = ../target/aarch64-apple-darwin/release
  else
  CXX = arch -x86_64 clang++ # for intel, we only use clang++
  TARGET_DIR = ../target/release
  endif
 else
  TARGET_DIR = ../target/release
 endif
else
 ifeq ($(shell uname -m), arm64)
  ifeq ($(UNAME), Darwin)
   ifeq ($(USE_ARM64),true)
  CXX = clang++  # for arm64, we only use clang++
  TARGET_DIR = ../target/release
   else
  CXX = arch -x86_64 clang++ # for intel, we only use clang++
  TARGET_DIR = ../target/x86_64-apple-darwin/release
   endif
  endif
 else
  TARGET_DIR = ../target/release
 endif
endif

ifeq ($(UNAME), Darwin)
	FLAGS=$(CXXFLAGS) -framework Security -framework CoreFoundation
endif

ifeq ($(UNAME), Linux)
	FLAGS=$(CXXFLAGS) -lpthread -ldl -lrt
endif

all: clean clone build
build: prepare static dynamic cmake
run: run_static run_dynamic run_examples

# TARGET is specified in ../Makefile
# for example,
# Run `make aarch64-linux-android`, `TARGET` is `aarch64-linux-android`
android_build: TARGET_DIR=../target/$(TARGET)/release
android_build: prepare

ios_build: TARGET_DIR=../target/$(TARGET)/release
ios_build: prepare

clone:
	git submodule update --init --recursive

prepare:
	python3 helper.py --target_dir $(TARGET_DIR)

easywsclient.o:
	$(CXX) -c third_party/easywsclient/easywsclient.cpp $(CXXFLAGS)

# wrapper of libplay_cpp_sdk.a
libplay_cpp_sdk.so:
ifeq ($(UNAME), Linux)
	$(CXX) -fPIC -shared -o $(sdk_dir)/lib/libplay_cpp_sdk.so \
		$(sdk_dir)/include/*.cc \
		$(sdk_dir)/include/extra-cpp-bindings/src/*.cc \
		$(sdk_dir)/include/defi-wallet-core-cpp/src/*.cc \
		$(sdk_dir)/lib/libcxxbridge1.a \
		$(sdk_dir)/lib/libplay_cpp_sdk.a \
		$(FLAGS)
endif

# add rpath to dylib
libplay_cpp_sdk.dylib:
ifeq ($(UNAME), Darwin)
	install_name_tool -id @rpath/libplay_cpp_sdk.dylib $(sdk_dir)/lib/libplay_cpp_sdk.dylib
endif

static: easywsclient.o
	mkdir -p bin
ifeq ($(MSYSTEM), MINGW64)
	$(CXX) -o bin/demostatic \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		$(sdk_dir)/lib/libplay_cpp_sdk.a \
		-lws2_32 -lntdll \
		-lcrypt32 -lsecur32 -lbcrypt -luserenv -lncrypt \
		-std=c++14 $(FLAGS) 
else
	$(CXX) -o bin/demostatic \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		$(sdk_dir)/lib/libplay_cpp_sdk.a \
		-std=c++14 $(FLAGS)
endif 

dynamic: easywsclient.o libplay_cpp_sdk.so libplay_cpp_sdk.dylib
	mkdir -p bin
ifeq ($(UNAME), Linux)
	$(CXX) -o bin/demo \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		$(sdk_dir)/lib/libplay_cpp_sdk.so \
		-std=c++14 $(FLAGS)
endif
ifeq ($(UNAME), Darwin)
	$(CXX) -o bin/demo \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		$(sdk_dir)/include/*.cc \
		$(sdk_dir)/include/extra-cpp-bindings/src/*.cc \
		$(sdk_dir)/include/defi-wallet-core-cpp/src/*.cc \
		$(sdk_dir)/lib/libcxxbridge1.a \
		$(sdk_dir)/lib/libplay_cpp_sdk.dylib \
		-rpath $(sdk_dir)/lib \
		-std=c++14 $(FLAGS)
endif

cmake:
# if mac and arm64 do nothing
ifeq ($(USE_ARM64),true)
	echo "skip cmake"
else 
	mkdir -p build
	cd build && CXX="$(CXX)" CXXFLAGS=$(CXXFLAGS) cmake .. && make
endif	
run_static:
	. ./.env && ./bin/demostatic && ./build/demostatic

run_dynamic:
	. ./.env && export LD_LIBRARY_PATH=$(PWD)/lib && ./bin/demo && ./build/demo

run_examples:
	./build/examples/new_wallet
	./build/examples/restore_wallet
	# . .env && ./build/examples/get_erc20_transfer_history_blocking
	# . .env && ./build/examples/get_erc721_transfer_history_blocking
	./build/examples/get_tokens_blocking
	./build/examples/get_token_transfers_blocking
	./build/examples/get_token_transfers_blocking
	./build/examples/wallet_connect
	./build/examples/unit
	./build/examples/get_wallet

clean:
	rm -rf easywsclient.o bin build

# clean-artifacts can not be run if user uses prebuilt binaries and bindings
clean-artifacts:
	rm -rf $(sdk_dir)/lib
	rm -rf $(sdk_dir)/include
