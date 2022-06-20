# Changelog

## [Unreleased]
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
