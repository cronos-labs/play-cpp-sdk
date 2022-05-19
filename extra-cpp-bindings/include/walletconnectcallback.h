#pragma once

#include "rust/cxx.h"
#include <memory>
namespace com {
namespace crypto {
namespace game_sdk {
using namespace rust;
 
    class WalletConnectSessionInfo {
        public:
        /// if the wallet approved the connection
        bool connected;
        /// hex-string(0x...), the accounts returned by the wallet
        Vec<String> accounts;
        /// u64, the chain id returned by the wallet
        String chain_id;
        /// the bridge server URL
        String bridge;
        /// the secret key used in encrypting wallet requests
        /// and decrypting wallet responses as per WalletConnect 1.0
        /// hex-string(0x...), 32 bytes 
        String key;
        /// this is the client's randomly generated ID
        String client_id;
        /// json, the client metadata (that will be presented to the wallet in the initial request)
        String client_meta;
        /// uuid, the wallet's ID
        String peer_id;
        /// json, the wallet's metadata
        String peer_meta;
        /// uuid, the one-time request ID
        String handshake_topic;

        void set_connected(bool connected) ;
        void set_accounts(Vec<String> accounts) ;
        void set_chainid(rust::String chainid) ;
        void set_bridge(String bridge) ;
        void set_key(String key) ;
        void set_clientid(String client_id) ;
        void set_clientmeta(String client_meta) ;
        void set_peerid(String client_id) ;
        void set_peermeta(String client_meta) ;
        void set_handshaketopic(String handshake_topic) ;
};  
std::unique_ptr<WalletConnectSessionInfo> new_walletconnect_sessioninfo();


class WalletConnectCallback {
    public:
    WalletConnectCallback();
    virtual ~WalletConnectCallback();
    void onConnected(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const;
    void onDisconnected(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const ;
    void onConnecting(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const ;
    void onUpdated(std::unique_ptr<WalletConnectSessionInfo> sessioninfo) const;
};

std::unique_ptr<WalletConnectCallback> new_walletconnect_callback();
}}}