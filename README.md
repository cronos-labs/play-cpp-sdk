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
3. For windows, Visual Studio 2019 or newer

# Build
## Windows
1. If you use Visual Studio 2022, open `demo.vcxproj` and upgrade PlatformToolset to v143.
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
