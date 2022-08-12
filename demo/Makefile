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
	TARGET_DIR = ../target/release
endif

ifeq ($(shell uname -m), arm64)
	CXX = arch -x86_64 clang++ # for arm64, we only use clang++
	TARGET_DIR = ../target/x86_64-apple-darwin/release
else

endif

ifeq ($(UNAME), Darwin)
	FLAGS=$(CXXFLAGS) -framework Security -framework CoreFoundation
endif

ifeq ($(UNAME), Linux)
	FLAGS=$(CXXFLAGS) -lpthread -ldl
endif

all: clean clone build
build: prepare static dynamic cmake
run: run_static run_dynamic run_examples

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
	$(CXX) -o bin/demostatic \
		easywsclient.o \
		main.cc \
		chainmain.cc \
		cronos.cc \
		extra.cc \
		$(sdk_dir)/lib/libplay_cpp_sdk.a \
		-std=c++14 $(FLAGS)

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
	mkdir -p build
	cd build && CXX=$(CXX) CXXFLAGS=$(CXXFLAGS) cmake .. && make

run_static:
	. ./.env && ./bin/demostatic && ./build/demostatic

run_dynamic:
	. ./.env && export LD_LIBRARY_PATH=$(PWD)/lib && ./bin/demo && ./build/demo

run_examples:
	./build/new_wallet
	./build/restore_wallet
	. .env && ./build/get_erc20_transfer_history_blocking
	. .env && ./build/get_erc721_transfer_history_blocking
	./build/get_tokens_blocking
	./build/get_token_transfers_blocking

clean:
	rm -rf easywsclient.o bin build

# clean-artifacts can not be run if user uses prebuilt binaries and bindings
clean-artifacts:
	rm -rf $(sdk_dir)/lib
	rm -rf $(sdk_dir)/include
