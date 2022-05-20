#include "include/extra-cpp-bindings/src/lib.rs.h"
#include "include/rust/cxx.h"
#include <cassert>
#include <chrono>
#include <iostream>
#include <thread>

using namespace std;
using namespace com::crypto::game_sdk;
using namespace rust;

String getEnv(String key) {
  String ret;
  if (getenv(key.c_str()) != nullptr) {
    ret = getenv(key.c_str());
  }
  return ret;
}

// Read CronoScan api key in env
const String CRONOSCAN_API_KEY = getEnv("CRONOSCAN_API_KEY");
// Read pay api key in env
const String PAY_API_KEY = getEnv("PAY_API_KEY");

int main(int argc, char *argv[]) {

  // CronoScan examples
  if (CRONOSCAN_API_KEY != "") {
    Vec<RawTxDetail> txs = get_transaction_history_blocking(
        "0x7de9ab1e6a60ac7a70ce96d1d95a0dfcecf7bfb7", CRONOSCAN_API_KEY);
    cout << txs.size() << endl;

    for (Vec<RawTxDetail>::iterator ptr = txs.begin(); ptr < txs.end(); ptr++) {
      cout << ptr->hash << " ";
      cout << ptr->to_address << " ";
      cout << ptr->from_address << " ";
      cout << ptr->value << " ";
      cout << ptr->block_no << " ";
      cout << ptr->timestamp << " ";
      cout << ptr->contract_address << " " << endl;
    }

    Vec<RawTxDetail> erc20_txs = get_erc20_transfer_history_blocking(
        "0xa9b34a4b568e640d5e5d1e6e13101025e1262864",
        "0x66e428c3f67a68878562e79A0234c1F83c208770",
        QueryOption::ByAddressAndContract, CRONOSCAN_API_KEY);

    for (Vec<RawTxDetail>::iterator ptr = erc20_txs.begin();
         ptr < erc20_txs.end(); ptr++) {
      cout << ptr->hash << " ";
      cout << ptr->to_address << " ";
      cout << ptr->from_address << " ";
      cout << ptr->value << " ";
      cout << ptr->block_no << " ";
      cout << ptr->timestamp << " ";
      cout << ptr->contract_address << " " << endl;
    }

    Vec<RawTxDetail> erc721_txs = get_erc721_transfer_history_blocking(
        "0x668f126b87936df4f9a98f18c44eb73868fffea0",
        "0xbd6b9a1A0477d64E99F660b7b7C205f4604E4Ff3", QueryOption::ByContract,
        CRONOSCAN_API_KEY);

    for (Vec<RawTxDetail>::iterator ptr = erc721_txs.begin();
         ptr < erc721_txs.end(); ptr++) {
      cout << ptr->hash << " ";
      cout << ptr->to_address << " ";
      cout << ptr->from_address << " ";
      cout << ptr->value << " ";
      cout << ptr->block_no << " ";
      cout << ptr->timestamp << " ";
      cout << ptr->contract_address << " " << endl;
    }
  }

  // Blockscout examples
  Vec<RawTokenResult> tokens_txs =
      get_tokens_blocking("https://blockscout.com/xdai/mainnet/api",
                          "0x652d53227d7013f3FbBeA542443Dc2eeF05719De");
  for (Vec<RawTokenResult>::iterator ptr = tokens_txs.begin();
       ptr < tokens_txs.end(); ptr++) {
    cout << ptr->balance << " ";
    cout << ptr->contract_address << " ";
    cout << ptr->decimals << " ";
    cout << ptr->id << " ";
    cout << ptr->name << " ";
    cout << ptr->symbol << " ";
    cout << ptr->token_type << endl;
  }

  Vec<RawTxDetail> token_transfer_txs = get_token_transfers_blocking(
      "https://cronos.org/explorer/testnet3/api",
      "0x841a15D12aEc9c6039FD132c2FbFF112eD355700", "", QueryOption::ByAddress);
  for (Vec<RawTxDetail>::iterator ptr = token_transfer_txs.begin();
       ptr < token_transfer_txs.end(); ptr++) {
    cout << ptr->hash << " ";
    cout << ptr->to_address << " ";
    cout << ptr->from_address << " ";
    cout << ptr->value << " ";
    cout << ptr->block_no << " ";
    cout << ptr->timestamp << " ";
    cout << ptr->contract_address << " " << endl;
  }

  // pay api examples
  try {
    OptionalArguments opiton_args;
    opiton_args.description = "Crypto.com Tee (Unisex)";
    CryptoComPaymentResponse resp =
        create_payment(PAY_API_KEY, "2500", "USD", opiton_args);
    cout << resp.id << " ";
    cout << resp.main_app_qr_code << " ";
    cout << resp.onchain_deposit_address << " ";
    cout << resp.base_amount << " ";
    cout << resp.currency << " ";
    cout << resp.expiration << " ";
    cout << resp.status << endl;

    resp = get_payment(PAY_API_KEY, resp.id);
    cout << resp.id << " ";
    cout << resp.main_app_qr_code << " ";
    cout << resp.onchain_deposit_address << " ";
    cout << resp.base_amount << " ";
    cout << resp.currency << " ";
    cout << resp.expiration << " ";
    cout << resp.status << endl;

  } catch (const rust::cxxbridge1::Error &e) {
    // Use `Assertion failed`, the same as `assert` function
    cout << "Assertion failed: " << e.what() << endl;
  }

  return 0;
}
