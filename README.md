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
3. Optional: GNU make
4. Optional: Visual Studio 2019 or newer for windows

# Pre-build Download
Please download the archive file based on your OS release:
https://github.com/crypto-com/play-cpp-sdk/releases

- Visual Studio 2019 MSVC, x86_64, toolset 14.29 or newer: `play_cpp_sdk_Windows_x86_64.zip`
- macOS 10.15 or newer: `play_cpp_sdk_Darwin_x86_64.tar.gz`
- Ubuntu 20.04 or newer: `play_cpp_sdk_Linux_x86_64.tar.gz`

## Setup in a demo Visual C++ project

### Windows
1. Clone the current repository
    ``` sh
    git clone https://github.com/crypto-com/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder
3. Open `demo.sln` which includes two projects: `demo` (dynamic build) and `demostatic` (static
   build). If you use Visual Studio 2022, retarget project, and upgrade PlatformToolset to
   v143.
4. Select `Release` profile.
5. Right click `demo` or `demostatic` project, click `Build` or `Rebuild` to build the project

#### Build Events
The following build events are included in the project file:
- Pre-Build event (`demo` and `demostatic`): `call pre_build.bat`
- Post-Build event (`demo`): `copy $(ProjectDir)lib\play_cpp_sdk.dll $(TargetDir)`

### Mac
1. Clone the current repository
    ``` sh
    git clone https://github.com/crypto-com/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder
3. Copy the dynamic library to `/usr/local/lib`
    ``` sh
    cd demo
    cp lib/libplay_cpp_sdk.dylib /usr/local/lib
    ```
4. Under `demo` folder and build the `demo` project
    ``` sh
    make
    ```

### Linux
1. Clone the current repository
    ``` sh
    git clone https://github.com/crypto-com/play-cpp-sdk.git
    ```
2. Unzip the archive file into `demo` folder
3. Under `demo` folder and build the `demo` project
    ``` sh
    make
    ```

## Setup in any other c++ 14 (or newer) projects
1. Unzip the archive file into the root folder of your project, you should see the following
   folders and files.
  - `include`: c++ source files and header files
  - `lib`: static and dynamic libraries
  - `CHANGELOG.md`
  - `LICENSE`

  We suggest:
  - In case of same name collision, we suggest you unzip the archive in a temporary folder and
  review them first.
  - Review the folder or file names under `include` and `lib` folder to see if there are files
  that have same names as in your project, rename those files in your project to avoid
  collision.
  - Finally copy `include` and `lib` folders into your project.
  - We will support CMAKE and provide you a better integration in future release.

2. Include the following headers and use the namespaces in your source codes
    ``` c++
    #include "include/defi-wallet-core-cpp/src/contract.rs.h" // erc20, erc721, erc1155 supports
    #include "include/defi-wallet-core-cpp/src/lib.rs.h" // wallet, EIP4361, query, signing, broadcast etc, on crypto.org and cronos
    #include "include/defi-wallet-core-cpp/src/nft.rs.h" // crypto.org chain nft support
    #include "include/defi-wallet-core-cpp/src/uint.rs.h" // uint256 type support
    #include "include/extra-cpp-bindings/src/lib.rs.h" // etherscan/cronoscan, crypto.com pay, wallet connect support
    #include "include/rust/cxx.h" // the important data types, e.g., rust::String, rust::str, etc

    using namespace rust;
    using namespace org::defi_wallet_core;
    using namespace com::crypto::game_sdk;
    ```
3. Link `play_cpp_sdk` static library in your build flow (check `demo/Makefile` for more
   details)
- Windows
    ``` c++
        lib\play_cpp_sdk.lib
    ```
- Mac or Linux
    ``` c++
        lib/libplay_cpp_sdk.a
    ```
4. Or link `play_cpp_sdk` dynamic library and `cxxbridge1` static library in your build flow
   (check `demo/Makefile` for more details)
- Windows
    ```
    lib\play_cpp_sdk.dll.lib
    lib\libcxxbridge1.a
    ```
- Mac
    ``` c++
    lib/libplay_cpp_sdk.dylib
    lib\libcxxbridge1.a
    ```
- Linux dynamic build is under testing.

# Build libraries and bindings from scratch
If the Pre-built release does not support your platform, you can build the binaries and
bindings on your own.

## Windows
1. Run `windows_build.bat` in x64 Native Tools Command Prompt for VS 2019. It will clone
   necessary submodules, build `play-cpp-sdk` crate, finally setup and build the demo project.
2. Clean `~/.cargo/git/checkouts` if cxx fails to build, then run `windows_build.bat` again.
3. Run `windows_install.bat`, libraries and bindings will be copied into a new created folder:
   `build`

### Notes about Visual Studio 2022
1. Open `demo.sln`. If you use Visual Studio 2022, retarget project, and upgrade
   PlatformToolset to `v143` before running `windows_build.bat`

## Mac
1. Run `make build_cpp` or `make cppx86_64`
2. Run `make install`, libraries and bindings will be copied into a new created folder: `build`

### Linux
1. Run `make build_cpp`
2. Run `make install`, libraries and bindings will be copied into a new created folder: `build`

### More information for Cronos Play
If you are a game developer, please visit [Cronos Play](https://cronos.org/play) or fill this [Contact Form](https://airtable.com/shrFiQnLrcpeBp2lS) for more information.
