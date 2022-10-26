#include <cassert>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <defi-wallet-core-cpp/src/uint.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
    rust::Box<Wallet> signer1_wallet = restore_wallet(
        "shed crumble dismiss loyal latin million oblige gesture "
        "shrug still oxygen custom remove ribbon disorder palace "
        "addict again blanket sad flock consider obey popular",
        "");
    rust::String signer1_address = signer1_wallet->get_eth_address(0);
    rust::Box<PrivateKey> signer1_privatekey =
        signer1_wallet->get_key("m/44'/60'/0'/0/0");

    rust::Box<Wallet> signer2_wallet =
        restore_wallet("night renew tonight dinner shaft scheme domain oppose "
                       "echo summer broccoli agent face guitar surface belt "
                       "veteran siren poem alcohol menu custom crunch index",
                       "");
    rust::String signer2_address = signer2_wallet->get_eth_address(0);
    rust::Box<PrivateKey> signer2_privatekey =
        signer2_wallet->get_key("m/44'/60'/0'/0/0");

    rust::String cronosrpc = "http://127.0.0.1:26651";

    // build transaction information
    EthTxInfoRaw eth_tx_info = new_eth_tx_info();
    eth_tx_info.to_address = signer2_address;
    eth_tx_info.nonce = get_eth_nonce(signer1_address, cronosrpc);
    eth_tx_info.amount = "1";
    eth_tx_info.amount_unit = EthAmount::EthDecimal;

    // build signed transaction
    rust::Vec<uint8_t> signedtx =
        build_eth_signed_tx(eth_tx_info, 777, true, *signer1_privatekey);
    U256 balance = get_eth_balance(signer1_address, cronosrpc);
    std::cout << "address=" << signer1_address
              << " balance=" << balance.to_string() << std::endl;

    // broadcast signed transaction
    rust::String status =
        broadcast_eth_signed_raw_tx(signedtx, cronosrpc, 1000).status;
    assert(status == "1");

    balance = get_eth_balance(signer1_address, cronosrpc);
    std::cout << "address=" << signer1_address
              << " balance=" << balance.to_string() << std::endl;

    return 0;
}
