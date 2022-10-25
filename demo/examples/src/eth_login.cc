#include <cassert>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
    // no \n in end of string
    std::string info =
        "service.org wants you to sign in with your Ethereum account:\n"
        "0xD09F7C8C4529CB5D387AA17E33D707C529A6F694\n"
        "\n"
        "I accept the ServiceOrg Terms of Service: https://service.org/tos\n"
        "\n"
        "URI: https://service.org/login\n"
        "Version: 1\n"
        "Chain ID: 1\n"
        "Nonce: 32891756\n"
        "Issued At: 2021-09-30T16:25:24Z\n"
        "Resources:\n"
        "- "
        "ipfs://bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq/\n"
        "- https://example.com/my-web2-claim.json";
    rust::Box<CppLoginInfo> logininfo = new_logininfo(info);

    rust::Box<Wallet> signer1_wallet = restore_wallet(
        "shed crumble dismiss loyal latin million oblige gesture "
        "shrug still oxygen custom remove ribbon disorder palace "
        "addict again blanket sad flock consider obey popular",
        "");
    rust::Box<PrivateKey> signer1_privatekey =
        signer1_wallet->get_key("m/44'/60'/0'/0/0");

    rust::String default_address =
        signer1_wallet->get_default_address(CoinType::CronosMainnet);
    rust::Vec<uint8_t> signature =
        logininfo->sign_logininfo(*signer1_privatekey);
    assert(signature.size() == 65);
    rust::Slice<const uint8_t> slice{signature.data(), signature.size()};
    logininfo->verify_logininfo(slice);
    return 0;
}
