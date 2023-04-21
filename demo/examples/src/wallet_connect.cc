#include <cassert>
#include <defi-wallet-core-cpp/src/contract.rs.h>
#include <defi-wallet-core-cpp/src/lib.rs.h>
#include <defi-wallet-core-cpp/src/uint.rs.h>
#include <extra-cpp-bindings/src/lib.rs.h> // nolint is not effective, it's compiler error, ignore
#include <fstream>
#include <iomanip>
#include <iostream>
#include <rust/cxx.h>
#include <sstream>
#include <chrono>
#include <thread>
using namespace com::crypto::game_sdk;
using namespace org::defi_wallet_core;

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
            rust::Vec<rust::String>(), "Defi WalletConnect Web3 Example",
            338); // ChainId of Cronos Testnet
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
    std::cout << "handshake_topic: " << sessioninfo.handshake_topic
              << std::endl;
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
    // exit(0);
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
        WalletConnectCallback *usercallbackraw =
            new UserWalletConnectCallback();
        std::unique_ptr<WalletConnectCallback> usercallback(usercallbackraw);
        client->setup_callback_blocking(std::move(usercallback));

        // Print the QR code on terminal
        rust::String uri = client->print_uri();

        // program is blocked here for waiting connecting
        WalletConnectEnsureSessionResult result =
            client->ensure_session_blocking();

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

        bool test_personal = false;
        bool test_basic = false;
        bool test_erc20 = true;

        // sign personal message
        if (test_personal) {
            /* message signing */
            rust::Vec<uint8_t> sig1 = client->sign_personal_blocking(
                "hello", result.addresses[0].address);
            std::cout << "signature=" << bytes_to_hex_string(sig1).c_str()
                      << std::endl;
            std::cout << "signature length=" << sig1.size() << std::endl;
        }

        // send transaction
        if (test_basic) {
            WalletConnectTxEip155 info;
            // send to the connected wallet itself
            // To send to other wallet address, simply
            // info.to = "0x....";
            info.to = rust::String(
                std::string("0x") +
                address_to_hex_string(result.addresses[0].address).c_str());
            info.value = "1000000000000000000"; // 1 TCRO
            info.common.chainid = result.chain_id;
            rust::Vec<uint8_t> receipt =
                client->send_eip155_transaction_blocking(
                    info, result.addresses[0].address);

            std::cout << "transaction_hash="
                      << bytes_to_hex_string(receipt).c_str() << std::endl;
        }

        // send contract transaction
        if (test_erc20) {
            WalletConnectTxCommon common;
            // Verify the contract
            // Test ERC20 Token: GLD
            // https://testnet.cronoscan.com/token/0xc213a7b37f4f7ec81f78895e50ea773aa8e78255
            Erc20 erc20 =
                new_erc20("0xC213a7B37F4f7eC81f78895E50EA773aA8E78255",
                          "https://evm-dev-t3.cronos.org", 338);
            assert(erc20.name() == "Gold");
            assert(erc20.symbol() == "GLD");
            assert(erc20.decimals() == 18);
            rust::String from_address = rust::String(
                std::string("0x") +
                address_to_hex_string(result.addresses[0].address).c_str());
            U256 erc20_balance = erc20.balance_of(from_address);
            std::cout << "erc20 balance=" << erc20_balance.to_string()
                      << std::endl;

            // construct tx info
            rust::String contract_action =
                R"({
                        "ContractTransfer": {
                            "Erc20Transfer": {
                                "contract_address": "0xC213a7B37F4f7eC81f78895E50EA773aA8E78255",
                                "to_address": "0xA914161b1b8d9dbC9c5310Fc7EBee5A5B18044b7",
                                "amount": "1000000000000000000"
                            }
                        }
                   })";

            common.chainid = result.chain_id;
            common.web3api_url =
                "https://evm-dev-t3.cronos.org"; // TODO unnessary for
                                                 // walletconnect

            rust::Vec<uint8_t> tx_hash = client->send_contract_transaction(
                contract_action, common, result.addresses[0].address);

            std::cout << "transaction_hash="
                      << bytes_to_hex_string(tx_hash).c_str() << std::endl;

            // TODO verify the balance is deducted, after transaction
            // successful
            // Workaround: sleep 3 second
            std::this_thread::sleep_for(std::chrono::seconds(3));
            assert(erc20.balance_of(from_address) ==
                   erc20_balance.sub(u256("1000000000000000000")));
        }

        // waiting update or disconnect
        while (true) {
            // sleep 1 second
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }

    } catch (const rust::Error e) {
        std::cout << "wallet connect error=" << e.what() << std::endl;
    }

    return 0;
}
