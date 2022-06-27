#include "extra-cpp-bindings/include/walletconnectcallback.h"
#include "extra-cpp-bindings/src/lib.rs.h"
#include <iostream>
#include <memory>

namespace com {
namespace crypto {
namespace game_sdk {

std::unique_ptr<WalletConnectSessionInfo> new_walletconnect_sessioninfo() {
  return std::make_unique<WalletConnectSessionInfo>();
}

void WalletConnectSessionInfo::set_connected(bool myconnected) {
  connected = myconnected;
}

void WalletConnectSessionInfo::set_chainid(rust::String mychainid) {
  chain_id = mychainid;
}

void WalletConnectSessionInfo::set_accounts(
    rust::Vec<rust::String> myaccounts) {
  accounts = myaccounts;
}

void WalletConnectSessionInfo::set_bridge(rust::String mybridge) {
  bridge = mybridge;
}

void WalletConnectSessionInfo::set_key(rust::String mykey) { key = mykey; }

void WalletConnectSessionInfo::set_clientid(rust::String myclient_id) {
  client_id = myclient_id;
}

void WalletConnectSessionInfo::set_clientmeta(rust::String myclient_meta) {
  client_meta = myclient_meta;
}

void WalletConnectSessionInfo::set_peerid(rust::String mypeer_id) {
  peer_id = mypeer_id;
}

void WalletConnectSessionInfo::set_peermeta(rust::String mypeer_meta) {
  peer_meta = mypeer_meta;
}

void WalletConnectSessionInfo::set_handshaketopic(
    rust::String myhandshake_topic) {
  handshake_topic = myhandshake_topic;
}

} // namespace game_sdk
} // namespace crypto
} // namespace com
