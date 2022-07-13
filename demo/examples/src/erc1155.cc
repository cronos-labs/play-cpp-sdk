#include <cassert>
#include <defi-wallet-core-cpp/src/contract.rs.h>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <defi-wallet-core-cpp/src/uint.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace org::defi_wallet_core;

int main(int argc, char *argv[]) {
  rust::Box<Wallet> signer1_wallet =
      restore_wallet("shed crumble dismiss loyal latin million oblige gesture "
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

  rust::Box<Wallet> validator1_wallet =
      restore_wallet("visit craft resemble online window solution west chuckle "
                     "music diesel vital settle comic tribe project blame bulb "
                     "armed flower region sausage mercy arrive release",
                     "");
  rust::String validator1_address = validator1_wallet->get_eth_address(0);
  rust::Box<PrivateKey> validator1_privatekey =
      validator1_wallet->get_key("m/44'/60'/0'/0/0");

  Erc1155 erc1155 = new_erc1155("0x939D7350c54228e4958e05b65512C4a5BB6A2ACc",
                                "http://127.0.0.1:26651", 777)
                        .legacy();
  // To be improved in the contract, now all uri are the same
  assert(erc1155.uri("0") == "https://game.example/api/item/{id}.json");
  assert(erc1155.uri("1") == "https://game.example/api/item/{id}.json");
  assert(erc1155.uri("2") == "https://game.example/api/item/{id}.json");
  assert(erc1155.uri("3") == "https://game.example/api/item/{id}.json");
  assert(erc1155.uri("4") == "https://game.example/api/item/{id}.json");
  assert(erc1155.balance_of(signer1_address, "0") ==
         u256("1000000000000000000"));
  assert(erc1155.balance_of(signer1_address, "1") ==
         u256("1000000000000000000000000000"));
  assert(erc1155.balance_of(signer1_address, "2") == u256("1"));
  assert(erc1155.balance_of(signer1_address, "3") == u256("1000000000"));
  assert(erc1155.balance_of(signer1_address, "4") == u256("1000000000"));

  // safe transfer erc1155 from signer1 to signer2
  rust::Vec<uint8_t> erc1155_data;
  rust::String status =
      erc1155.interval(3000)
          .safe_transfer_from(signer1_address, signer2_address, "0", "150",
                              erc1155_data, *signer1_privatekey)
          .status;
  assert(status == "1");
  assert(erc1155.balance_of(signer1_address, "0") ==
         u256("999999999999999850"));

  // safe batch transfer erc1155 from signer1 to signer2
  rust::Vec<rust::String> token_ids, amounts;
  token_ids.push_back("1");
  token_ids.push_back("2");
  token_ids.push_back("3");
  token_ids.push_back("4");

  amounts.push_back("200");
  amounts.push_back("1");
  amounts.push_back("300");
  amounts.push_back("400");
  status =
      erc1155
          .safe_batch_transfer_from(signer1_address, signer2_address, token_ids,
                                    amounts, erc1155_data, *signer1_privatekey)
          .status;
  assert(status == "1");
  assert(erc1155.balance_of(signer1_address, "1") ==
         u256("999999999999999999999999800"));
  assert(erc1155.balance_of(signer1_address, "2") == u256("0"));
  assert(erc1155.balance_of(signer1_address, "3") == u256("999999700"));
  assert(erc1155.balance_of(signer1_address, "4") == u256("999999600"));

  // toggle set_approval_for_all
  assert(erc1155.is_approved_for_all(signer1_address, signer2_address) == 0);
  erc1155.set_approval_for_all(signer2_address, true, *signer1_privatekey);
  assert(erc1155.is_approved_for_all(signer1_address, signer2_address) == 1);
  erc1155.set_approval_for_all(signer2_address, false, *signer1_privatekey);
  assert(erc1155.is_approved_for_all(signer1_address, signer2_address) == 0);
  // set approval for signer2
  erc1155.set_approval_for_all(signer2_address, true, *signer1_privatekey);
  assert(erc1155.is_approved_for_all(signer1_address, signer2_address) == 1);
  token_ids.clear();
  token_ids.push_back("1");
  token_ids.push_back("3");
  token_ids.push_back("4");

  amounts.clear();
  amounts.push_back("500");
  amounts.push_back("600");
  amounts.push_back("700");
  // and safe batch transfer from signer1 to validator1
  status = erc1155
               .safe_batch_transfer_from(signer1_address, validator1_address,
                                         token_ids, amounts, erc1155_data,
                                         *signer2_privatekey)
               .status;
  assert(status == "1");
  assert(erc1155.balance_of(signer1_address, "1") ==
         u256("999999999999999999999999300"));
  assert(erc1155.balance_of(signer1_address, "2") == u256("0"));
  assert(erc1155.balance_of(signer1_address, "3") == u256("999999100"));
  assert(erc1155.balance_of(signer1_address, "4") == u256("999998900"));

  assert(erc1155.balance_of(signer2_address, "1") == u256("200"));
  assert(erc1155.balance_of(signer2_address, "2") == u256("1"));
  assert(erc1155.balance_of(signer2_address, "3") == u256("300"));
  assert(erc1155.balance_of(signer2_address, "4") == u256("400"));

  assert(erc1155.balance_of(validator1_address, "1") == u256("500"));
  assert(erc1155.balance_of(validator1_address, "2") == u256("0"));
  assert(erc1155.balance_of(validator1_address, "3") == u256("600"));
  assert(erc1155.balance_of(validator1_address, "4") == u256("700"));

  return 0;
}
