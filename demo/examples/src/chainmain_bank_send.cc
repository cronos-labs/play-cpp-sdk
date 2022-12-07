#include <cassert>
#include <chrono>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
#include <thread>
using namespace org::defi_wallet_core;

const uint64_t GAS_LIMIT = 5000000;
const uint64_t FEE_AMOUNT = 25000000000;
const uint32_t COIN_TYPE = 394;

CosmosSDKTxInfoRaw
build_txinfo() { // NOLINT(modernize-use-trailing-return-type)
    CosmosSDKTxInfoRaw ret;
    ret.account_number = 0;
    ret.sequence_number = 0;
    ret.gas_limit = GAS_LIMIT;
    ret.fee_amount = FEE_AMOUNT;
    ret.fee_denom = "basecro";
    ret.timeout_height = 0;
    ret.memo_note = "";
    ret.chain_id = "chainmain-1";
    ret.coin_type = COIN_TYPE;
    ret.bech32hrp = "cro";
    return ret;
}

int main(int argc, char *argv[]) { // NOLINT(modernize-use-trailing-return-type)
    CosmosSDKTxInfoRaw tx_info = build_txinfo();
    rust::String from_address = "cro1u08u5dvtnpmlpdq333uj9tcj75yceggszxpnsy";
    rust::String to_address = "cro1apdh4yc2lnpephevc6lmpvkyv6s5cjh652n6e4";
    rust::String grpc = "http://127.0.0.1:26803";
    rust::String servercosmos = "http://127.0.0.1:26804";
    rust::String servertendermint = "http://127.0.0.1:26807";
    rust::Box<Wallet> wallet = restore_wallet(
        "shed crumble dismiss loyal latin million oblige gesture "
        "shrug still oxygen custom remove ribbon disorder palace "
        "addict again blanket sad flock consider obey popular",
        "");

    // check the original balance
    rust::String balance = query_account_balance(grpc, to_address, "basecro");
    std::cout << "balance=" << balance << std::endl;

    // query account detils
    rust::String detailjson = query_account_details(servercosmos, from_address);
    std::cout << "detailjson=" << detailjson << std::endl;

    // update account_number and sequence_number after querying account details
    // info
    CosmosAccountInfoRaw detailinfo =
        query_account_details_info(servercosmos, from_address);
    tx_info.account_number = detailinfo.account_number;
    tx_info.sequence_number = detailinfo.sequence_number;

    // get the private key
    rust::Box<PrivateKey> privatekey = wallet->get_key("m/44'/394'/0'/0/0");

    // transfer 1 basecro
    rust::Vec<uint8_t> signedtx = get_single_bank_send_signed_tx(
        tx_info, *privatekey, to_address, 1, "basecro");
    CosmosTransactionReceiptRaw resp = broadcast_tx(servertendermint, signedtx);
    std::cout << "tx_hash_hex: " << resp.tx_hash_hex << std::endl
              << "code: " << resp.code << std::endl
              << "log: " << resp.log << std::endl;

    // dealy and make sure the block is updated
    std::this_thread::sleep_for(std::chrono::seconds(3));

    // check balance updated
    balance = query_account_balance(grpc, to_address, "basecro");
    std::cout << "balance=" << balance << std::endl;
}
