# Compile with g++: make CXX=g++
# Compile with clang++: make CXX=clang++
# Compile with default compiler: make
UNAME := $(shell uname)

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

# Comment out to set your ndk version
# r21e, for unreal 4.27 and 5
# NDK_VERSION=21.4.7075529
# the newest ndk version
# NDK_VERSION=25.1.8937393
#
# or set NDK_VERSION on command line
# NDK_VERSION=21.4.7075529 make armv7-linux-androideabi
# NDK_VERSION=21.4.7075529 make aarch64-linux-android
# NDK_VERSION=21.4.7075529 make i686-linux-android
# NDK_VERSION=21.4.7075529 make x86_64-linux-android

# The ndk requirement for unreal, please check
# https://docs.unrealengine.com/4.27/en-US/SharingAndReleasing/Mobile/Android/AndroidSDKRequirements/
# https://docs.unrealengine.com/5.0/en-US/android-development-requirements-for-unreal-engine/
#
# NDK releases >= 23 beta3 no longer include libgcc which rust's pre-built
# standard libraries depend on. As a workaround for newer NDKs we redirect
# libgcc to libunwind.
# See https://github.com/rust-lang/rust/pull/85806
# TODO Here we only check 21 or non-21
ifneq (,$(findstring 21,$(NDK_VERSION)))
	RUSTFLAGS +=
else
	RUSTFLAGS +="-L$(shell pwd)/env/android"
endif

# RUSTFLAGS +=
ifeq ($(UNAME), Darwin)
# Please install NDK via Android Studio
	NDK_HOME=$(HOME)/Library/Android/sdk/ndk/$(NDK_VERSION)
	TOOLCHAIN=$(NDK_HOME)/toolchains/llvm/prebuilt/darwin-x86_64
endif

ifeq ($(UNAME), Linux)
# Change NDK_HOME if necessary
	NDK_HOME=/usr/local/lib/android/sdk/ndk/$(NDK_VERSION)
	TOOLCHAIN=$(NDK_HOME)/toolchains/llvm/prebuilt/linux-x86_64
endif

# Set this to your minSdkVersion.
API=21

all: build_cpp

clone:
	git submodule update --init --recursive

build_play-cpp-sdk: clone
ifeq ($(shell uname -m), x86_64)
ifeq ($(UNAME), Darwin)
	MACOSX_DEPLOYMENT_TARGET=10.15 CXX=$(CXX) CXXFLAGS=$(CXXFLAGS) cargo build --package play-cpp-sdk --release
endif
ifeq ($(UNAME), Linux)
	CXX=$(CXX) CXXFLAGS=$(CXXFLAGS) cargo build --package play-cpp-sdk --release
endif
endif
ifeq ($(shell uname -m), arm64)
	rustup target add x86_64-apple-darwin
	MACOSX_DEPLOYMENT_TARGET=10.15 CXX=$(CXX) CXXFLAGS=$(CXXFLAGS) cargo build --package play-cpp-sdk --release --target x86_64-apple-darwin
endif

build_extra-cpp-bindings:
	CXX=$(CXX) CXXFLAGS=$(CXXFLAGS) cargo build --package extra-cpp-bindings --release

build_cpp: build_play-cpp-sdk
	MACOSX_DEPLOYMENT_TARGET=10.15 && cd demo && make build

armv7-linux-androideabi: TARGET=armv7-linux-androideabi
armv7-linux-androideabi: android_armv7
	cd demo && TARGET=$(TARGET) make android_build

aarch64-linux-android: TARGET=aarch64-linux-android
aarch64-linux-android: android_aarch64
	cd demo && TARGET=$(TARGET) make android_build

i686-linux-android: TARGET=i686-linux-android
i686-linux-android: android_i686
	cd demo && TARGET=$(TARGET) make android_build

x86_64-linux-android: TARGET=x86_64-linux-android
x86_64-linux-android: android_x86_64
	cd demo && TARGET=$(TARGET) make android_build

aarch64-apple-ios: TARGET=aarch64-apple-ios
aarch64-apple-ios: ios_aarch64
	cd demo && TARGET=$(TARGET) make ios_build

android_armv7:
	rustup target add $(TARGET)
	TARGET_CC=$(TOOLCHAIN)/bin/armv7a-linux-androideabi$(API)-clang \
	CXX=$(TOOLCHAIN)/bin/armv7a-linux-androideabi$(API)-clang++ \
	TARGET_AR=$(TOOLCHAIN)/bin/llvm-ar \
	CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$(TOOLCHAIN)/bin/armv7a-linux-androideabi$(API)-clang \
	RUSTFLAGS=$(RUSTFLAGS) cargo build --target=$(TARGET) --package play-cpp-sdk --release

android_aarch64:
	rustup target add $(TARGET)
	TARGET_CC=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	CXX=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang++ \
	TARGET_AR=$(TOOLCHAIN)/bin/llvm-ar \
	CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	RUSTFLAGS=$(RUSTFLAGS) cargo build --target=$(TARGET) --package play-cpp-sdk --release

android_i686:
	rustup target add $(TARGET)
	TARGET_CC=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	CXX=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang++ \
	TARGET_AR=$(TOOLCHAIN)/bin/llvm-ar \
	CARGO_TARGET_I686_LINUX_ANDROID_LINKER=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	RUSTFLAGS=$(RUSTFLAGS) cargo build --target=$(TARGET) --package play-cpp-sdk --release

android_x86_64:
	rustup target add $(TARGET)
	TARGET_CC=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	CXX=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang++ \
	TARGET_AR=$(TOOLCHAIN)/bin/llvm-ar \
	CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=$(TOOLCHAIN)/bin/$(TARGET)$(API)-clang \
	RUSTFLAGS=$(RUSTFLAGS) cargo build --target=$(TARGET) --package play-cpp-sdk --release

ios_aarch64:
	rustup target add $(TARGET)
	cargo build --target=$(TARGET) --package play-cpp-sdk --release

cpp: build_cpp
# 1. In order to use crypto pay api, you need to Generate Keys in
# https://merchant.crypto.com/developers/api_keys first
#
# 2. Copy the `Publishable Key` or `Secret Key` as `PAY_API_KEY`'s value in `.env`
# cd demo && git submodule update --init --recursive && make build
	cd demo && make run

cpp-ci-tests: build_cpp
	./integration_test.sh

webhook:
# 1. Install ngrok for crypto pay api testing: https://ngrok.com/download
#
# 2. Run `ngrok http 4567` in a seperate terminal first, then add the `payload url` into
# https://merchant.crypto.com/developers/webhooks
#
# 3. Find the `SIGNATURE SECRET` in merchant dashboard, and copy it as
# `PAY_WEBHOOK_SIGNATURE_SECRET`'s value in `.env`
	cd demo && . ./.env && npm install && node server.js

install:
	. ./install.sh

uninstall:
	rm -rf install
