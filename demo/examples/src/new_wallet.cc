#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
  rust::Box<Wallet> wallet = new_wallet("", MnemonicWordCount::TwentyFour);
  std::cout << wallet->get_default_address(CoinType::CronosMainnet)
            << std::endl;
  std::cout << wallet->get_address(CoinType::CronosMainnet, 0) << std::endl;
  std::cout << wallet->get_eth_address(0) << std::endl;
  rust::Box<PrivateKey> private_key = wallet->get_key("m/44'/60/0'/0/0");
  return 0;
}
