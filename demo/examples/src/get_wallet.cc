#include <cassert>
#include <chrono>
#include <extra-cpp-bindings/src/lib.rs.h> // nolint is not effective, it's compiler error, ignore
#include <fstream>
#include <iomanip>
#include <iostream>
#include <rust/cxx.h>
#include <sstream>
#include <thread>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {

    // empty path, cache is false
    // Fetch new listings without saving to local file
    rust::Vec<WalletEntry> wallet_entries = get_all_wallets(false, "");

    // non-empty path, cache is false
    // Fetch new listings and save to local file
    wallet_entries = get_all_wallets(false, "wallets.json");

    // empty path, cache is true
    // Load the hardcoded listings
    wallet_entries = get_all_wallets(true, "");

    // non-empty path, cache is true
    // Load the listings from a local file
    wallet_entries = get_all_wallets(true, "wallets.json");

    wallet_entries = filter_wallets(true, "", Platform::Mobile);
    assert(wallet_entries.size() == 164);
    std::cout << "Total mobile Wallets: " << wallet_entries.size() << std::endl;
    for (const WalletEntry &entry : wallet_entries) {
        std::cout << "Mobile | ";
        std::cout << entry.id << " | ";
        std::cout << entry.name << " | ";
        // std::cout << entry.image_url.sm << " ";
        // std::cout << entry.image_url.md << " ";
        // std::cout << entry.image_url.lg << " ";
        std::cout << entry.mobile_native_link << " | ";
        std::cout << entry.mobile_universal_link << " " << std::endl;
    }

    wallet_entries = filter_wallets(true, "", Platform::Desktop);
    assert(wallet_entries.size() == 20);
    for (const WalletEntry &entry : wallet_entries) {
        std::cout << "Desktop | ";
        std::cout << entry.id << " | ";
        std::cout << entry.name << " | ";
        // std::cout << entry.image_url.sm << " ";
        // std::cout << entry.image_url.md << " ";
        // std::cout << entry.image_url.lg << " ";
        std::cout << entry.desktop_native_link << " | ";
        std::cout << entry.desktop_universal_link << " " << std::endl;
    }

    auto id =
        "f2436c67184f158d1beda5df53298ee84abfc367581e4505134b5bcf5f46697d";
    assert(check_wallet(true, "", id, Platform::Mobile) == true);
    assert(check_wallet(true, "", id, Platform::Desktop) == true);
    WalletEntry wallet = get_wallet(true, "", id);
    assert(wallet.name == "Crypto.com | DeFi Wallet");
    assert(wallet.mobile_native_link == "cryptowallet:");
    assert(wallet.mobile_universal_link == "https://wallet.crypto.com");
    assert(wallet.desktop_native_link == "cryptowallet:");
    assert(wallet.desktop_universal_link == "");

    id = "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96";
    assert(check_wallet(true, "", id, Platform::Mobile) == true);
    assert(check_wallet(true, "", id, Platform::Desktop) == false);
    wallet = get_wallet(true, "", id);
    assert(wallet.name == "MetaMask");
    assert(wallet.mobile_native_link == "metamask:");
    assert(wallet.mobile_universal_link == "https://metamask.app.link");
    assert(wallet.desktop_native_link == "");
    assert(wallet.desktop_universal_link == "");

    id = "4622a2b2d6af1c9844944291e5e7351a6aa24cd7b23099efac1b2fd875da31a0";
    assert(check_wallet(true, "", id, Platform::Mobile) == true);
    assert(check_wallet(true, "", id, Platform::Desktop) == false);
    wallet = get_wallet(true, "", id);
    assert(wallet.name == "Trust Wallet");
    assert(wallet.mobile_native_link == "trust:");
    assert(wallet.mobile_universal_link == "https://link.trustwallet.com");
    assert(wallet.desktop_native_link == "");
    assert(wallet.desktop_universal_link == "");

    return 0;
}
