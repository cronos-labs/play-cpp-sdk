#!/usr/bin/env bash
# Please notice: some env, for example, CRONOSCAN_API_KEY, PAY_API_KEY, and PAY_WEBSOCKET_PORT
# will be loaded in test.yml
#
# Or you can edit `demo/.env` then run `source demo/.env` to load them

# Set up `CPP_EXAMPLE_PATH` for cpp integration test
PWD=$(pwd)
export CPP_EXAMPLE_PATH=$PWD/demo/bin/demostatic
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/chainmain_bank_send
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/chainmain_nft
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/uint
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/eth
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/eth_login
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/erc20
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/erc721
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

export CPP_EXAMPLE_PATH=$PWD/demo/build/examples/erc1155
nix-shell defi-wallet-core-rs/integration_tests/shell.nix --run defi-wallet-core-rs/scripts/python-tests

$PWD/demo/build/examples/get_erc20_transfer_history_blocking
$PWD/demo/build/examples/get_erc721_transfer_history_blocking
$PWD/demo/build/examples/get_tokens_blocking
$PWD/demo/build/examples/get_token_transfers_blocking
$PWD/demo/build/examples/create_payment
$PWD/demo/build/examples/wallet_connect
$PWD/demo/build/examples/new_wallet
$PWD/demo/build/examples/restore_wallet
