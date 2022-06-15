#include "include/defi-wallet-core-cpp/src/lib.rs.h"
#include "include/rust/cxx.h"
#include <iostream>
using namespace org::defi_wallet_core;
using namespace std;

int main(int argc, char *argv[]) {
    auto wallet = restore_wallet("shed crumble dismiss loyal latin million oblige gesture shrug still oxygen custom remove ribbon disorder palace addict again blanket sad flock consider obey popular", "");
    cout << wallet->get_default_address(CoinType::CronosMainnet) << endl;
    cout << wallet->get_address(CoinType::CronosMainnet, 0) << endl;
    cout << wallet->get_eth_address(0) << endl;
    auto private_key = wallet->get_key("m/44'/60/0'/0/0");
    return 0;
}
