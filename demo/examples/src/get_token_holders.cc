#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

int main(int argc, char *argv[]) {
    rust::Vec<TokenHolderDetail> token_holders =
        get_token_holders("https://blockscout.com/xdai/mainnet/api",
                          "0xed1efc6efceaab9f6d609fec89c9e675bf1efb0a", 1, 100);
    for (const TokenHolderDetail &tx : token_holders) {
        std::cout << tx.address << " ";
        std::cout << tx.value << " " << std::endl;
    }

    return 0;
}
