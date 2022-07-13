#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {
  // Blockscout examples
  rust::Vec<RawTokenResult> tokens_txs =
      get_tokens_blocking("https://blockscout.com/xdai/mainnet/api",
                          "0x652d53227d7013f3FbBeA542443Dc2eeF05719De");
  for (const RawTokenResult &tx : tokens_txs) {
    std::cout << tx.balance << " ";
    std::cout << tx.contract_address << " ";
    std::cout << tx.decimals << " ";
    std::cout << tx.id << " ";
    std::cout << tx.name << " ";
    std::cout << tx.symbol << " ";
    std::cout << tx.token_type << std::endl;
  }

  return 0;
}
