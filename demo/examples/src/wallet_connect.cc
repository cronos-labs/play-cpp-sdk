#include <cassert>
#include <extra-cpp-bindings/src/lib.rs.h>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <rust/cxx.h>
#include <sstream>
using namespace com::crypto::game_sdk;

// convert byte array to hex string
rust::String bytes_to_hex_string(rust::Vec<uint8_t> bytes) {
  std::stringstream ret;
  ret << std::hex;
  for (int i = 0; i < bytes.size(); i++) {
    ret << std::setw(2) << std::setfill('0') << (int)bytes[i];
  }
  return ret.str();
}

rust::String address_to_hex_string(::std::array<::std::uint8_t, 20> bytes) {
  std::stringstream ret;
  ret << std::hex;
  for (int i = 0; i < 20; i++) {
    ret << std::setw(2) << std::setfill('0') << (int)bytes[i];
  }
  return ret.str();
}

// if session already exists, restore session
rust::Box<WalletconnectClient> make_new_client(std::string filename) {

  std::ifstream file(filename.c_str());
  if (file.is_open()) {
    std::string sessioninfostring((std::istreambuf_iterator<char>(file)),
                                  std::istreambuf_iterator<char>());
    rust::Box<WalletconnectClient> client =
        walletconnect_restore_client(sessioninfostring);
    return client;
  } else {
    rust::Box<WalletconnectClient> client = walletconnect_new_client(
        "Defi WalletConnect example.", "http://localhost:8080/",
        rust::Vec<rust::String>(), "Defi WalletConnect Web3 Example");
    std::cout << "qrcode= " << client->get_connection_string() << std::endl;

    return client;
  }
}

class UserWalletConnectCallback : public WalletConnectCallback {
public:
  UserWalletConnectCallback() {}
  virtual ~UserWalletConnectCallback() {}
  void onConnected(const WalletConnectSessionInfo &sessioninfo) const;
  void onDisconnected(const WalletConnectSessionInfo &sessioninfo) const;
  void onConnecting(const WalletConnectSessionInfo &sessioninfo) const;
  void onUpdated(const WalletConnectSessionInfo &sessioninfo) const;
};
void print_session(const WalletConnectSessionInfo &sessioninfo) {
  std::cout << "connected: " << sessioninfo.connected << std::endl;
  std::cout << "chain_id: " << sessioninfo.chain_id << std::endl;
  // iterate over accounts
  for (auto &account : sessioninfo.accounts) {
    std::cout << "account: " << account << std::endl;
  }
  std::cout << "bridge: " << sessioninfo.bridge << std::endl;
  std::cout << "client_id: " << sessioninfo.client_id << std::endl;
  std::cout << "client_meta: " << sessioninfo.client_meta << std::endl;
  std::cout << "peer_id: " << sessioninfo.peer_id << std::endl;
  std::cout << "peer_meta: " << sessioninfo.peer_meta << std::endl;
  std::cout << "handshake_topic: " << sessioninfo.handshake_topic << std::endl;
}
void UserWalletConnectCallback::onConnected(
    const WalletConnectSessionInfo &sessioninfo) const {
  std::cout << "user c++ onConnected" << std::endl;
  print_session(sessioninfo);
}
void UserWalletConnectCallback::onDisconnected(
    const WalletConnectSessionInfo &sessioninfo) const {
  std::cout << "user c++ onDisconnected" << std::endl;
  print_session(sessioninfo);
  exit(0);
}
void UserWalletConnectCallback::onConnecting(
    const WalletConnectSessionInfo &sessioninfo) const {
  std::cout << "user c++ onConnecting" << std::endl;
  print_session(sessioninfo);
  // !!! Important !!!
  // Comment out this line for actual test
  exit(0);
}
void UserWalletConnectCallback::onUpdated(
    const WalletConnectSessionInfo &sessioninfo) const {
  std::cout << "user c++ onUpdated" << std::endl;
  print_session(sessioninfo);
}

int main(int argc, char *argv[]) {
  std::string filename = "sessioninfo.json";
  try {
    rust::Box<WalletconnectClient> client = make_new_client(filename);
    WalletConnectCallback *usercallbackraw = new UserWalletConnectCallback();
    std::unique_ptr<WalletConnectCallback> usercallback(usercallbackraw);
    client->setup_callback(std::move(usercallback));

    // Print the QR code on terminal
    rust::String uri = client->print_uri();

    // program is blocked here for waiting connecting
    WalletConnectEnsureSessionResult result = client->ensure_session_blocking();

    // once connected, program continues
    std::cout << "connected chain_id: " << result.chain_id << std::endl;
    assert(result.addresses.size() > 0);

    // get the connected session info as string and save it into a file
    rust::String sessioninfo = client->save_client();
    std::cout << "sessioninfo = " << sessioninfo << std::endl;
    std::ofstream outfile(filename);
    outfile.write(sessioninfo.c_str(), sessioninfo.size());
    // it is important to close file and release the session file
    outfile.close();

    // sign
    bool test_personal = true;
    if (test_personal) {
      /* message signing */
      rust::Vec<uint8_t> sig1 =
          client->sign_personal_blocking("hello", result.addresses[0].address);
      std::cout << "signature=" << bytes_to_hex_string(sig1).c_str()
                << std::endl;
      std::cout << "signature length=" << sig1.size() << std::endl;
    } else {
      /* legacy eth sign */
      WalletConnectTxLegacy info;
      info.to = rust::String(
          std::string("0x") +
          address_to_hex_string(result.addresses[0].address).c_str());
      info.gas = "21000";             // gas limit
      info.gas_price = "10000";       // gas price
      info.value = "100000000000000"; // 0.0001 eth
      info.data = rust::Vec<uint8_t>();
      info.nonce = "1";
      rust::Vec<uint8_t> sig1 = client->sign_legacy_transaction_blocking(
          info, result.addresses[0].address);

      std::cout << "signature=" << bytes_to_hex_string(sig1).c_str()
                << std::endl;
      std::cout << "signature length=" << sig1.size() << std::endl;
    }

    // waiting update or disconnect
    while (true) {
    }

  } catch (const rust::Error e) {
    std::cout << "wallet connect error=" << e.what() << std::endl;
  }

  return 0;
}
