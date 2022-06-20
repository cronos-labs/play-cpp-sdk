# Cronos Play C++ SDK

This project includes the following crates:
- `play-cpp-sdk`: the cpp sdk wrapper
- `defi-wallet-core-rs`: a dependency of play-cpp-sdk
- `extra-cpp-bindings`: a dependency of play-cpp-sdk
- `wallet-connect`: wallet connect implementation

## Security Warning

No security audits of this project have ever been performed.

The project is still in development and is *alpha* quality.

USE AT YOUR OWN RISK!

# Requirements
1. python 3.8 or newer
2. rust 1.61 or newer
3. GNU make
4. For windows, Visual Studio 2019 or newer

# Pre-build Download
Please download the archive file based on your os version: https://github.com/crypto-com/play-cpp-sdk/releases

## Windows
1. Unzip the archive file into `demo` folder
2. Open `demo.sln` which includes two projects: `demo` (dynamic build) and `demostatic` (static
   build). If you use Visual Studio 2022, retarget project, and upgrade PlatformToolset to
   v143.
3. Select `Release` profile.
4. Right click `demo` or `demostatic` project, click `Build` or `Rebuild` to build the project

### Build Events
Pre-Build event: `call pre_build.bat`
Post-Build event (dynamic build): `copy $(ProjectDir)lib\play_cpp_sdk.dll $(TargetDir)`

## Mac
1. Unzip the archive file into `demo` folder
2. In order to use the dynamic library correctly, please copy the dynamic library to `/usr/local/lib`
    ``` sh
    cd demo
    cp lib/libplay_cpp_sdk.dylib /usr/local/lib
    ```
4. Build the `demo` project
    ``` sh
    make
    ```
## Linux
1. Unzip the archive file into `demo` folder
2. Build the `demo` project
    ``` sh
    make
    ```

# Build from scratch
## Windows
1. Open `demo.sln`. If you use Visual Studio 2022, retarget project, and upgrade
   PlatformToolset to v143.
2. Run `windows_build.bat` in x64 native command prompt. It will clone necessary submodules,
   build `play-cpp-sdk` crate, finally setup and build the demo project.
3. Clean `~/.cargo/git/checkouts` if cxx fails to build.

## Mac
`make cpp` or `make cppx86_64`

### Linux
`make cpp`

# wallet-connect
This crate contains the WalletConnect 1.0 client implementation the could be used by dApps in integrations.

## WalletConnect 1.0
For protocol details, see the technical specification: https://docs.walletconnect.com/tech-spec

## Usage
See "examples/web3.rs". The WalletConnect client implements the [ethers middleware](https://docs.rs/ethers/latest/ethers/providers/struct.Provider.html),
so one can call the Web3 JSON-RPC API methods: https://docs.walletconnect.com/json-rpc-api-methods/ethereum
after the client is linked with the external wallet.

You can use https://test.walletconnect.org/ for testing (not for production).

## Implementation
The implementation code is largely based off the unfinished WalletConnect Rust Client: https://github.com/nlordell/walletconnect-rs
The following major changes were made:
- The websocket implementation (originally `ws`) was replaced with `tokio-tungstenite` for better portability and compatibility with the async ecosystem.
- The cryptographic implementation (originally using `openssl`) was replaced with the [RustCrypto](https://github.com/RustCrypto) implementations in pure Rust
(given they are used elsewhere in the codebase as well as in major dependencies).
- The Ethereum transport implementation (originally using `web3`) was replaced with the `ethers` which is used elsewhere in the codebase. The extensibility of `ethers` allowed more Web3 methods to be reused.
