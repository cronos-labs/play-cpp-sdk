#include <cassert>
#include <defi-wallet-core-cpp/src/contract.rs.h>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <defi-wallet-core-cpp/src/uint.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
    rust::Box<Wallet> signer1_wallet = restore_wallet(
        "shed crumble dismiss loyal latin million oblige gesture "
        "shrug still oxygen custom remove ribbon disorder palace "
        "addict again blanket sad flock consider obey popular",
        "");
    rust::String signer1_address = signer1_wallet->get_eth_address(0);
    rust::Box<PrivateKey> signer1_privatekey =
        signer1_wallet->get_key("m/44'/60'/0'/0/0");

    rust::Box<Wallet> signer2_wallet =
        restore_wallet("night renew tonight dinner shaft scheme domain oppose "
                       "echo summer broccoli agent face guitar surface belt "
                       "veteran siren poem alcohol menu custom crunch index",
                       "");
    rust::String signer2_address = signer2_wallet->get_eth_address(0);
    rust::Box<PrivateKey> signer2_privatekey =
        signer2_wallet->get_key("m/44'/60'/0'/0/0");

    rust::Box<Wallet> validator1_wallet = restore_wallet(
        "visit craft resemble online window solution west chuckle "
        "music diesel vital settle comic tribe project blame bulb "
        "armed flower region sausage mercy arrive release",
        "");
    rust::String validator1_address = validator1_wallet->get_eth_address(0);
    rust::Box<PrivateKey> validator1_privatekey =
        validator1_wallet->get_key("m/44'/60'/0'/0/0");

    Erc721 erc721 = new_erc721("0x2305f3980715c9D247455504080b41072De38aB9",
                               "http://127.0.0.1:26651", 777)
                        .legacy();
    assert(erc721.name() == "GameItem");
    assert(erc721.symbol() == "ITM");
    assert(erc721.token_uri("1") == "https://game.example/item-id-8u5h2m.json");
    // cout << "Total Supply of ERC721=" << erc721.total_supply() << endl; //
    // the contract must support IERC721Enumerable
    assert(erc721.owner_of("1") == signer1_address);
    assert(erc721.balance_of(signer1_address) == u256("1"));

    // transfer erc721 from signer1 to signer2
    rust::String status = erc721
                              .transfer_from(signer1_address, signer2_address,
                                             "1", *signer1_privatekey)
                              .status;
    assert(status == "1");
    assert(erc721.balance_of(signer1_address) == u256("0"));
    assert(erc721.owner_of("1") == signer2_address);

    // safe transfer erc721 from signer2 to signer1
    status = erc721
                 .safe_transfer_from(signer2_address, signer1_address, "1",
                                     *signer2_privatekey)
                 .status;
    assert(status == "1");
    assert(erc721.balance_of(signer1_address) == u256("1"));
    assert(erc721.owner_of("1") == signer1_address);

    assert(erc721.balance_of(signer1_address) == u256("1"));
    assert(erc721.get_approved("1") ==
           "0x0000000000000000000000000000000000000000");
    // toggle set_approval_for_all
    assert(erc721.is_approved_for_all(signer1_address, signer2_address) == 0);
    erc721.set_approval_for_all(signer2_address, true, *signer1_privatekey);
    assert(erc721.is_approved_for_all(signer1_address, signer2_address) == 1);
    erc721.set_approval_for_all(signer2_address, false, *signer1_privatekey);
    assert(erc721.is_approved_for_all(signer1_address, signer2_address) == 0);

    // signer1 approve singer2 to transfer erc721
    erc721.approve(signer2_address, "1", *signer1_privatekey);
    assert(erc721.get_approved("1") == signer2_address);

    // safe transfer erc721 from signer1 to validator1
    status = erc721
                 .safe_transfer_from(signer1_address, validator1_address, "1",
                                     *signer2_privatekey)
                 .status;
    assert(status == "1");
    assert(erc721.balance_of(validator1_address) == u256("1"));
    assert(erc721.owner_of("1") == validator1_address);

    // validator1 set_approval_for_all for singer2 to transfer all assets
    assert(erc721.is_approved_for_all(validator1_address, signer2_address) ==
           0);
    erc721.set_approval_for_all(signer2_address, true, *validator1_privatekey);
    assert(erc721.is_approved_for_all(validator1_address, signer2_address) ==
           1);
    // safe transfer erc721 from validator1 to signer1
    status = erc721
                 .safe_transfer_from(validator1_address, signer1_address, "1",
                                     *signer2_privatekey)
                 .status;
    assert(status == "1");
    assert(erc721.balance_of(signer1_address) == u256("1"));
    assert(erc721.owner_of("1") == signer1_address);

    return 0;
}
