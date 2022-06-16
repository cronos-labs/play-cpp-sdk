# Cronos Play C++ SDK

This project includes the following crates:
- `play-cpp-sdk`: the cpp sdk wrapper
- `defi-wallet-core-rs`: a dependency of play-cpp-sdk
- `extra-cpp-bindings`: a dependency of play-cpp-sdk
- `wallet-connect`: wallet connect implementation

# Build
## Windows
1. Install Visual Studio 2019 or newer
2. Run `windows_build.bat` in x64 native command prompt. It will build all the things and setup the demo project

### Notes on windows
1. If you use Visual Studio 2022, you need to upgrade the PlatformToolset of `demo.vcxproj` to v143
2. Only static lib `play-cpp-sdk.lib` is supported, linking `play-cpp-sdk.dll.lib` is not supported at this moment.
3. Clean `~/.cargo/git/checkouts` if cxx fails to build.

## Mac or Linux
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
