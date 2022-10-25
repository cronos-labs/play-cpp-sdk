#pragma once

#include "rust/cxx.h"
#include <memory>
namespace com {
namespace crypto {
namespace game_sdk {

class WalletConnectSessionInfo {
  public:
    /// if the wallet approved the connection
    bool connected;
    /// hex-string(0x...), the accounts returned by the wallet
    rust::Vec<rust::String> accounts;
    /// u64, the chain id returned by the wallet
    rust::String chain_id;
    /// the bridge server URL
    rust::String bridge;
    /// the secret key used in encrypting wallet requests
    /// and decrypting wallet responses as per WalletConnect 1.0
    /// hex-string(0x...), 32 bytes
    rust::String key;
    /// this is the client's randomly generated ID
    rust::String client_id;
    /// json, the client metadata (that will be presented to the wallet in the
    /// initial request)
    rust::String client_meta;
    /// uuid, the wallet's ID
    rust::String peer_id;
    /// json, the wallet's metadata
    rust::String peer_meta;
    /// uuid, the one-time request ID
    rust::String handshake_topic;

    void set_connected(bool connected);
    void set_accounts(rust::Vec<rust::String> accounts);
    void set_chainid(rust::String chainid);
    void set_bridge(rust::String bridge);
    void set_key(rust::String key);
    void set_clientid(rust::String client_id);
    void set_clientmeta(rust::String client_meta);
    void set_peerid(rust::String client_id);
    void set_peermeta(rust::String client_meta);
    void set_handshaketopic(rust::String handshake_topic);
};
std::unique_ptr<WalletConnectSessionInfo> new_walletconnect_sessioninfo();

class WalletConnectCallback {
  public:
    virtual ~WalletConnectCallback() {} // need virtual to prevent memory leak
    // need to pure virtual to prevent incorrect callback
    virtual void
    onConnected(const WalletConnectSessionInfo &sessioninfo) const = 0;
    virtual void
    onDisconnected(const WalletConnectSessionInfo &sessioninfo) const = 0;
    virtual void
    onConnecting(const WalletConnectSessionInfo &sessioninfo) const = 0;
    virtual void
    onUpdated(const WalletConnectSessionInfo &sessioninfo) const = 0;
};

std::unique_ptr<WalletConnectCallback> new_walletconnect_callback();
} // namespace game_sdk
} // namespace crypto
} // namespace com
