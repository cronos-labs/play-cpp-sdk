# Changelog
## [Unreleased]
## [v0.0.11-alpha] - 2022-11-14
- Add IOS build
- Update defi-wallet-core-cpp to v0.3.0

## [v0.0.10-alpha] - 2022-11-09
- Change erc-20,erc-721,erc-1155 tx to eip-155
- Convert message to hex before being sent for walletconnect personal_sign function
- Added cpp-lint, semgrep and codeql analysis for C++

## [v0.0.9-alpha] - 2022-11-01
- Add optional field chain_id for walletconnect (In C++, 0 = None)
- Add wallet connect with contract calls (modified client to be cloneable)

## [v0.0.8-alpha] - 2022-09-13
- Add missing licenses
- Fix QR code can not be detected error
- Rename `setup_callback` as `setup_callback_blocking`

## [v0.0.7-alpha] - 2022-08-24
- Add android builds

## [v0.0.6-alpha] - 2022-08-17
- Add checksum for linux libc++ release
- Add qrcode api

## [v0.0.5-alpha] - 2022-08-12
- Support both g++ and clang
- Add libc++ build for linux Unreal plugin

## [v0.0.4-alpha] - 2022-08-10
Add get-backup-mnemonics, generate-mnemonics

## [v0.0.3-alpha] - 2022-08-01
Mac release to support 10.15
Fix unicode decode error on windows 11
Update ethers and cxx-build

## [v0.0.2-alpha] - 2022-07-18
### Security Warning
No security audits of this release have ever been performed yet.

The project is still in development and is alpha quality.

USE AT YOUR OWN RISK!

### Added
- Add CMake support
- Add `get_token_holders` function
- Add examples

### Changed
- Replace openssl with `rustls`
- Improve dynamic build for mac (change to rapth using `install_name_tool`) and linux (build a
dynamic library wrapper on a static library)
- Replace the `cargo test` execution with `cargo llvm-cov`
- Replace `grpc-web-client` with `tonic-web-wasm-client`

### Removed

## [v0.0.1-alpha] - 2022-06-21
### Security Warning
No security audits of this release have ever been performed yet.

The project is still in development and is alpha quality.

USE AT YOUR OWN RISK!

### Added
- Add `play-cpp-sdk` crate for building static or dynamic libraries and providing bindings
  (headers and sources) for c++ projects
- Add [defi-wallet-core-rs](https://github.com/crypto-com/defi-wallet-core-rs) as submodule,
  and one of dependencies of `play-cpp-sdk` crate
- Add `extra-cpp-bindings` as one of dependencies of `play-cpp-sdk` crate
  - Add Etherscan/Cronoscan function `get_transaction_history_blocking` for acquiring the
  transactions of a given address
  - Add Etherscan/Cronoscan function `get_erc20_transfer_history_blocking` for getting the
  ERC20 transfers of a given address of a given contract
  - Add Etherscan/Cronoscan function `get_erc721_transfer_history_blocking` for getting the
  ERC721 transfers of a given address of a given contract
  - Add BlockScout function `get_tokens_blocking` returning the list of all owned tokens
  - Add BlockScout function `get_token_transfers_blocking` returning all the token transfers
  - Add Crypto.com Pay functions `create_payment` and `get_payment`
  - Add WalletConnect support
    - Add wallet connect function `walletconnect_new_client` to create opaque pointer for wallet-connect
    - Add wallet connect function `setup_callback` to setup callbacks for wallet-connect events
    - Add wallet connect function `ensure_session_blocking` to ensure that wallet-connnect session is created or restored
    - Add wallet connect function `get_connection_string` to get string for qrcode generation
    - Add wallet connect function `sign_personal_blocking` to sign general message
    - Add wallet connect function `sign_legacy_transaction_blocking` to sign legacy transaction
- Add a `demo` kick-starter project: link, build, and test the apis of the cpp sdk.
