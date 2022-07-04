#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {
  rust::Vec<RawTxDetail> token_transfer_txs = get_token_transfers_blocking(
      "https://cronos.org/explorer/testnet3/api",
      "0x841a15D12aEc9c6039FD132c2FbFF112eD355700", "", QueryOption::ByAddress);
  for (rust::Vec<RawTxDetail>::iterator ptr = token_transfer_txs.begin();
       ptr < token_transfer_txs.end(); ptr++) {
    std::cout << ptr->hash << " ";
    std::cout << ptr->to_address << " ";
    std::cout << ptr->from_address << " ";
    std::cout << ptr->value << " ";
    std::cout << ptr->block_no << " ";
    std::cout << ptr->timestamp << " ";
    std::cout << ptr->contract_address << " " << std::endl;
  }

  return 0;
}
