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
  rust::Vec<WalletEntry> wallet_entries =
      get_all_wallets(false, "");

  // non-empty path, cache is false
  // Fetch new listings and save to local file
  wallet_entries = get_all_wallets(false, "wallets.json");

  // empty path, cache is true
  // Load the hardcoded listings
  wallet_entries = get_all_wallets(true, "");

  // non-empty path, cache is true
  // Load the listings from a local file
  wallet_entries = get_all_wallets(true, "wallets.json");

  wallet_entries =
      filter_wallets(true, "", Platform::Mobile);

  for (const WalletEntry &entry : wallet_entries) {
    std::cout << entry.id << " | ";
    std::cout << entry.name << " | ";
    // std::cout << entry.image_url.sm << " ";
    // std::cout << entry.image_url.md << " ";
    // std::cout << entry.image_url.lg << " ";
    std::cout << entry.mobile_native_link << " | ";
    std::cout << entry.mobile_universal_link << " " << std::endl;
  }

  wallet_entries = filter_wallets(true, "", Platform::Desktop);

  for (const WalletEntry &entry : wallet_entries) {
    std::cout << entry.id << " | ";
    std::cout << entry.name << " | ";
    // std::cout << entry.image_url.sm << " ";
    // std::cout << entry.image_url.md << " ";
    // std::cout << entry.image_url.lg << " ";
    std::cout << entry.desktop_native_link << " | ";
    std::cout << entry.desktop_universal_link << " " << std::endl;
  }

  WalletEntry wallet = get_wallet(
      true, "",
      "f2436c67184f158d1beda5df53298ee84abfc367581e4505134b5bcf5f46697d");

  assert(wallet.name == "Crypto.com | DeFi Wallet");

  wallet = get_wallet(
      true, "",
      "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96");

  assert(wallet.name == "MetaMask");

  wallet = get_wallet(
      true, "",
      "4622a2b2d6af1c9844944291e5e7351a6aa24cd7b23099efac1b2fd875da31a0");

  assert(wallet.name == "Trust Wallet");

  return 0;
}
