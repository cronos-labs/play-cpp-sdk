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

  Erc20 erc20 = new_erc20("0x5003c1fcc043D2d81fF970266bf3fa6e8C5a1F3A",
                          "http://127.0.0.1:26651", 777)
                    .legacy();
  assert(erc20.name() == "Gold");
  assert(erc20.symbol() == "GLD");
  assert(erc20.decimals() == 18);
  U256 erc20_total_supply = erc20.total_supply();
  assert(erc20_total_supply == u256("100000000000000000000000000"));
  U256 erc20_balance = erc20.balance_of(signer1_address);
  assert(erc20_balance == erc20_total_supply);

  // transfer erc20 token from signer1 to signer2
  rust::String status =
      erc20.transfer(signer2_address, "100", *signer1_privatekey).status;
  assert(status == "1");
  assert(erc20.balance_of(signer1_address) == erc20_balance.sub(u256("100")));

  // signer1 approve singer2 allowance
  erc20.interval(3000).approve(signer2_address, "1000", *signer1_privatekey);
  rust::String allowance = erc20.allowance(signer1_address, signer2_address);
  assert(allowance == "1000");

  // transfer from signer1 to validator1 using the allowance mechanism
  erc20.transfer_from(signer1_address, validator1_address, "100",
                      *signer2_privatekey);
  allowance = erc20.allowance(signer1_address, signer2_address);
  assert(allowance == "900");

  return 0;
}
