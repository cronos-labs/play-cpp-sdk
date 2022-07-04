#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {
  // Blockscout examples
  rust::Vec<RawTokenResult> tokens_txs =
      get_tokens_blocking("https://blockscout.com/xdai/mainnet/api",
                          "0x652d53227d7013f3FbBeA542443Dc2eeF05719De");
  for (rust::Vec<RawTokenResult>::iterator ptr = tokens_txs.begin();
       ptr < tokens_txs.end(); ptr++) {
    std::cout << ptr->balance << " ";
    std::cout << ptr->contract_address << " ";
    std::cout << ptr->decimals << " ";
    std::cout << ptr->id << " ";
    std::cout << ptr->name << " ";
    std::cout << ptr->symbol << " ";
    std::cout << ptr->token_type << std::endl;
  }

  return 0;
}
