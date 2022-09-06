# Cronos Play C++ SDK

This project includes the following crates:
- `play-cpp-sdk`: the cpp sdk wrapper
- `defi-wallet-core-rs`: a dependency of play-cpp-sdk
- `extra-cpp-bindings`: a dependency of play-cpp-sdk
- `wallet-connect`: wallet connect implementation

## Security Warning

No security audits of this project have ever been performed yet.

The project is still in development and is *alpha* quality.

USE AT YOUR OWN RISK!

# Requirements
1. python 3.8 or newer
2. rust 1.61 or newer
3. C++ 14 or newer
4. Optional: CMake
5. Optional: GNU make for mac and linux, ninja for windows
6. Optional: Visual Studio 2019 or newer for windows

# Pre-built Download
Please download the archive file based on your OS release:
https://github.com/cronos-labs/play-cpp-sdk/releases

- Visual Studio 2019 MSVC, x86_64, toolset 14.29 or newer: `play_cpp_sdk_Windows_x86_64.zip`
- macOS 10.15 or newer: `play_cpp_sdk_Darwin_x86_64.tar.gz`
- Ubuntu 20.04 or newer: `play_cpp_sdk_libstdc++_Linux_x86_64.tar.gz` or `play_cpp_sdk_libc++_Linux_x86_64.tar.gz`
- Android: `play_cpp_sdk_$(TARGET)-$(NDK_VERSION).tar.gz`

## Setup a demo project
### Windows
#### Visual Studio Project
Start with a C++ project with `.sln` and `.vcxproj` files:
1. Clone the current repository
    ``` sh
    git clone https://github.com/cronos-labs/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder, and replace the original `sdk` folder
3. Open `demo.sln` which includes two projects: `demo` (dynamic build) and `demostatic` (static
   build). If you use Visual Studio 2022, retarget project, and upgrade PlatformToolset to
   v143.
4. Select `Release` profile.
5. Right click `demo` or `demostatic` project, click `Build` or `Rebuild` to build the project

#### CMake Project
Build modern, cross-platform C++ apps that don't depend on `.sln` or `.vcxproj` files:
1. Open Visual Studio, then open a local folder in welcome window (or click `File` > `Open` >
   `Folder...` in the menu), locate the `demo` folder and open it
2. Select configuration `x64-Release` in the tool bar
3. Click `Build` > `Build All` or `Rebuild All` to build the project

### Mac
1. Clone the current repository
    ``` sh
    git clone https://github.com/cronos-labs/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder, and replace the original `sdk` folder
3. Under `demo` folder and build the `demo` project
    ``` sh
    make CXX=g++     # Compile with g++
    make CXX=clang++ # Compile with clang++
    make             # Compile with default compiler
    ```

### Linux
1. Clone the current repository
    ``` sh
    git clone https://github.com/cronos-labs/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder, and replace the original `sdk` folder
3. Under `demo` folder and build the `demo` project
    ``` sh
    make CXX=g++     # Compile with g++
    make CXX=clang++ # Compile with clang++
    make             # Compile with default compiler
    ```

## Setup a c++ 14 (or newer) project
1. Unzip the archive file into the root folder of your project, you should see a folder named `sdk` and its subdirectories/files.
   ```
    - sdk
      - CMakeLists.txt
      - include: c++ source files and header files
      - lib: static and dynamic libraries
      - CHANGELOG.md
      - LICENSE
   ```

2. Include the following headers and use the namespaces in your source codes based on your need
    ``` c++
    #include "sdk/include/defi-wallet-core-cpp/src/contract.rs.h" // erc20, erc721, erc1155 supports
    #include "sdk/include/defi-wallet-core-cpp/src/lib.rs.h" // wallet, EIP4361, query, signing, broadcast etc, on crypto.org and cronos
    #include "sdk/include/defi-wallet-core-cpp/src/nft.rs.h" // crypto.org chain nft support
    #include "sdk/include/defi-wallet-core-cpp/src/uint.rs.h" // uint256 type support
    #include "sdk/include/extra-cpp-bindings/src/lib.rs.h" // etherscan/cronoscan, crypto.com pay, wallet connect support
    #include "sdk/include/rust/cxx.h" // the important data types, e.g., rust::String, rust::str, etc

    using namespace rust;
    using namespace org::defi_wallet_core;
    using namespace com::crypto::game_sdk;
    ```
3. Link the `play_cpp_sdk` static or dynamic library, `cxxbridge1` static library, and sources
   (*.cc) into your build system (Visual Studio solution, CMake or Makefile). For more details,
   check out [Cronos Play Docs](https://github.com/crypto-org-chain/cronos-play-docs).

# Build libraries and bindings from scratch
If the Pre-built release does not support your platform, you can build the binaries and
bindings on your own.

## Windows
1. Run `windows_build.bat` in x64 Native Tools Command Prompt for VS 2019. It will clone
   necessary submodules, build `play-cpp-sdk` crate, finally setup and build the demo project.
2. Clean `~/.cargo/git/checkouts` if cxx fails to build, then run `windows_build.bat` again.
3. Run `windows_install.bat`, libraries and bindings will be copied into a new created folder:
   `install`

### Notes about Visual Studio 2022
1. Open `demo.sln`. If you use Visual Studio 2022, retarget project, and upgrade
   PlatformToolset to `v143` before running `windows_build.bat`

## Mac
1. Run `make`
2. Run `make install`, libraries and bindings will be copied into a new created folder: `install`

### Linux
1. Run `make`
2. Run `make install`, libraries and bindings will be copied into a new created folder: `install`

### Android
1. Install android NDK (e.g. 21.4.7075529) via Android Studio
2. Run make for one of the following android targets on Mac or Linux
``` sh
NDK_VERSION=21.4.7075529 make armv7-linux-androideabi
NDK_VERSION=21.4.7075529 make aarch64-linux-android
NDK_VERSION=21.4.7075529 make i686-linux-android
NDK_VERSION=21.4.7075529 make x86_64-linux-android
```
3. Run `make install`, libraries and bindings will be copied into a new created folder: `install`

### More information for Cronos Play
If you are a game developer, please visit [Cronos Play](https://cronos.org/play) or fill this [Contact Form](https://airtable.com/shrFiQnLrcpeBp2lS) for more information.
