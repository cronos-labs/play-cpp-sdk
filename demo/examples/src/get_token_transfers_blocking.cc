#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {
  rust::Vec<RawTxDetail> token_transfer_txs = get_token_transfers_blocking(
      "https://cronos.org/explorer/testnet3/api",
      "0x841a15D12aEc9c6039FD132c2FbFF112eD355700", "", QueryOption::ByAddress);
  for (const RawTxDetail &tx : token_transfer_txs) {
    std::cout << tx.hash << " ";
    std::cout << tx.to_address << " ";
    std::cout << tx.from_address << " ";
    std::cout << tx.value << " ";
    std::cout << tx.block_no << " ";
    std::cout << tx.timestamp << " ";
    std::cout << tx.contract_address << " " << std::endl;
  }

  return 0;
}
