#include <cassert>
#include <chrono>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
#include <thread>
using namespace org::defi_wallet_core;

CosmosSDKTxInfoRaw build_txinfo() {
    CosmosSDKTxInfoRaw ret;
    ret.account_number = 0;
    ret.sequence_number = 0;
    ret.gas_limit = 5000000;
    ret.fee_amount = 25000000000;
    ret.fee_denom = "basecro";
    ret.timeout_height = 0;
    ret.memo_note = "";
    ret.chain_id = "chainmain-1";
    ret.coin_type = 394;
    ret.bech32hrp = "cro";
    return ret;
}

int main(int argc, char *argv[]) {
    CosmosSDKTxInfoRaw tx_info = build_txinfo();
    rust::String from = "cro1u08u5dvtnpmlpdq333uj9tcj75yceggszxpnsy";
    rust::String to = "cro1apdh4yc2lnpephevc6lmpvkyv6s5cjh652n6e4";
    rust::String servercosmos = "http://127.0.0.1:26804";
    rust::String servertendermint = "http://127.0.0.1:26807";
    rust::Box<Wallet> wallet = restore_wallet(
        "shed crumble dismiss loyal latin million oblige gesture "
        "shrug still oxygen custom remove ribbon disorder palace "
        "addict again blanket sad flock consider obey popular",
        "");

    // check the original balance
    rust::String balance =
        query_account_balance(servercosmos, to, "basecro", 1);
    std::cout << "balance=" << balance << std::endl;

    // query account detils
    rust::String detailjson = query_account_details(servercosmos, from);
    std::cout << "detailjson=" << detailjson << std::endl;

    // update account_number and sequence_number after querying account details
    // info
    CosmosAccountInfoRaw detailinfo =
        query_account_details_info(servercosmos, from);
    tx_info.account_number = detailinfo.account_number;
    tx_info.sequence_number = detailinfo.sequence_number;

    // get the private key
    rust::Box<PrivateKey> privatekey = wallet->get_key("m/44'/394'/0'/0/0");

    // transfer 1 basecro
    rust::Vec<uint8_t> signedtx =
        get_single_bank_send_signed_tx(tx_info, *privatekey, to, 1, "basecro");
    CosmosTransactionReceiptRaw resp = broadcast_tx(servertendermint, signedtx);
    std::cout << "tx_hash_hex: " << resp.tx_hash_hex << std::endl
              << "code: " << resp.code << std::endl
              << "log: " << resp.log << std::endl;

    // dealy and make sure the block is updated
    std::this_thread::sleep_for(std::chrono::seconds(3));

    // check balance updated
    balance = query_account_balance(servercosmos, to, "basecro", 1);
    std::cout << "balance=" << balance << std::endl;
}
