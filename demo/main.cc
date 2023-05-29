#include "chainmain.h"
#include "cronos.h"
#include "extra.h"
#include "sdk/include/defi-wallet-core-cpp/src/lib.rs.h"
#include "sdk/include/defi-wallet-core-cpp/src/nft.rs.h"
#include "sdk/include/extra-cpp-bindings/src/lib.rs.h"
#include "sdk/include/rust/cxx.h"
#include "third_party/easywsclient/easywsclient.hpp"
#include "third_party/json/single_include/nlohmann/json.hpp"
#include <atomic>
#include <cassert>
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <thread>

int main(int argc, char *argv[]) {
    try {
        chainmain_process();   // chain-main
        test_chainmain_nft();  // chainmain nft tests
        test_login();          // decentralized login
        cronos_process();      // cronos
        test_cronos_testnet(); // cronos testnet
        test_interval();
        test_blackscout_cronoscan();
        test_wallet_connect();
    } catch (const std::exception &e) {
        // Use `Assertion failed`, the same as `assert` function
        std::cout << "Assertion failed: " << e.what() << std::endl;
    }
    return 0;
}
