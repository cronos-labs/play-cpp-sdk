# Changelog

## [Unreleased]
### Added
- Add `play-cpp-sdk` crate for building static library `play_cpp_sdk.lib` and providing
  bindings (headers and sources) for c++ projects
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
  - Add wallet connect support
- Add a `demo` kick-starter project: link, build, and test the apis of the cpp sdk.
