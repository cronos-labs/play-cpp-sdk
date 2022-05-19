#include "extra-cpp-bindings/include/walletconnectcallback.h"
#include "extra-cpp-bindings/src/lib.rs.h"
#include <iostream>
#include <memory>
using namespace com::crypto::game_sdk;
using namespace rust;
using namespace std;

namespace com {
namespace crypto {
namespace game_sdk {

std::unique_ptr<WalletConnectSessionInfo> new_walletconnect_sessioninfo() {
  return std::make_unique<WalletConnectSessionInfo>();
}

void WalletConnectSessionInfo::set_connected(bool myconnected) 
{
      connected=myconnected;
}

void WalletConnectSessionInfo::set_chainid(rust::String mychainid) 
{
      chain_id=mychainid;
}

void WalletConnectSessionInfo::set_accounts(Vec<String> myaccounts) 
{
      accounts=myaccounts;
}

void WalletConnectSessionInfo::set_bridge(String mybridge) 
{
      bridge=mybridge;
}

void WalletConnectSessionInfo::set_key(String mykey) 
{
      key=mykey;
}

void WalletConnectSessionInfo::set_clientid(String myclient_id) 
{
      client_id=myclient_id;
}

void WalletConnectSessionInfo::set_clientmeta(String myclient_meta) 
{
      client_meta=myclient_meta;
}

void WalletConnectSessionInfo::set_peerid(String mypeer_id) 
{
      peer_id=mypeer_id;
}

void WalletConnectSessionInfo::set_peermeta(String mypeer_meta) 
{
      peer_meta=mypeer_meta;
}

void WalletConnectSessionInfo::set_handshaketopic(String myhandshake_topic) 
{
      handshake_topic=myhandshake_topic;
}

WalletConnectCallback::WalletConnectCallback() {
  std::cout << "WalletConnectCallback created" << std::endl;
}

WalletConnectCallback::~WalletConnectCallback() {}

void print_session(std::unique_ptr<WalletConnectSessionInfo>& sessioninfo)
{
      // print sessioninfo
      std::cout << "connected: " << sessioninfo->connected << std::endl;
      std::cout << "chain_id: " << sessioninfo->chain_id << std::endl;
      // iterate over accounts
      for (auto& account : sessioninfo->accounts) {
        std::cout << "account: " << account << std::endl;
      }
      std::cout << "bridge: " << sessioninfo->bridge << std::endl;
      std::cout << "client_id: " << sessioninfo->client_id << std::endl;
      std::cout << "client_meta: " << sessioninfo->client_meta << std::endl;
      std::cout << "peer_id: " << sessioninfo->peer_id << std::endl;
      std::cout << "peer_meta: " << sessioninfo->peer_meta << std::endl;
      std::cout << "handshake_topic: " << sessioninfo->handshake_topic << std::endl;

}
void WalletConnectCallback::onConnected(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const {
  cout << "c++ onConnected"  << endl;
  print_session(sessioninfo);
}
void WalletConnectCallback::onDisconnected(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const {
  cout << "c++ onDisconnected" << endl;
  print_session(sessioninfo);
}
void WalletConnectCallback::onConnecting(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const {
  cout << "c++ onConnecting" << endl;
  print_session(sessioninfo);
}
void WalletConnectCallback::onUpdated(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const {
  cout << "c++ onUpdated"  << endl;
  print_session(sessioninfo);
}

std::unique_ptr<WalletConnectCallback> new_walletconnect_callback() {
  return std::make_unique<WalletConnectCallback>();
}

} // namespace game_sdk
} // namespace crypto
} // namespace com