#include "chainmain.h"
#include "cronos.h"
#include "extra.h"
#include "include/defi-wallet-core-cpp/src/lib.rs.h"
#include "include/defi-wallet-core-cpp/src/nft.rs.h"
#include "include/extra-cpp-bindings/src/lib.rs.h"
#include "include/rust/cxx.h"
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
    } catch (const rust::cxxbridge1::Error &e) {
        // Use `Assertion failed`, the same as `assert` function
        std::cout << "Assertion failed: " << e.what() << std::endl;
    }

    test_interval();

    test_blackscout_cronoscan();
    test_wallet_connect();

    return 0;
}
