#include "extra.h"
#include "sdk/include/defi-wallet-core-cpp/src/lib.rs.h"
#include "sdk/include/defi-wallet-core-cpp/src/nft.rs.h"
#include "sdk/include/extra-cpp-bindings/src/lib.rs.h"
#include "sdk/include/rust/cxx.h"
#include "third_party/easywsclient/easywsclient.hpp"
#include "third_party/json/single_include/nlohmann/json.hpp"
#include <cassert>
#include <atomic>
#include <cassert>
#include <chrono>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <sstream>
#include <thread>

using namespace std;
using namespace com::crypto::game_sdk;
using namespace rust;
using namespace nlohmann;

void test_crypto_pay();
void websocket_client_thread(std::atomic<bool> &stop_thread, String &id);

inline String getEnv(String key);

inline String getEnv(String key) {
    String ret;
    if (getenv(key.c_str()) != nullptr) {
        ret = getenv(key.c_str());
    }
    return ret;
}

// Read CronoScan api key in env
const String CRONOSCAN_API_KEY = getEnv("CRONOSCAN_API_KEY");
// Read pay api key in env
const String PAY_API_KEY = getEnv("PAY_API_KEY");
// Read websocket port in env
const String PAY_WEBSOCKET_PORT = getEnv("PAY_WEBSOCKET_PORT");

void test_blackscout_cronoscan() {
    // CronoScan examples
    if (CRONOSCAN_API_KEY != "") {
        Vec<RawTxDetail> txs = get_transaction_history_blocking(
            "0x7de9ab1e6a60ac7a70ce96d1d95a0dfcecf7bfb7", CRONOSCAN_API_KEY);
        std::cout << txs.size() << endl;

        for (Vec<RawTxDetail>::iterator ptr = txs.begin(); ptr < txs.end();
             ptr++) {
            std::cout << ptr->hash << " ";
            std::cout << ptr->to_address << " ";
            std::cout << ptr->from_address << " ";
            std::cout << ptr->value << " ";
            std::cout << ptr->block_no << " ";
            std::cout << ptr->timestamp << " ";
            std::cout << ptr->contract_address << " " << endl;
        }

        Vec<RawTxDetail> erc20_txs = get_erc20_transfer_history_blocking(
            "0xa9b34a4b568e640d5e5d1e6e13101025e1262864",
            "0x66e428c3f67a68878562e79A0234c1F83c208770",
            QueryOption::ByAddressAndContract, CRONOSCAN_API_KEY);

        for (Vec<RawTxDetail>::iterator ptr = erc20_txs.begin();
             ptr < erc20_txs.end(); ptr++) {
            std::cout << ptr->hash << " ";
            std::cout << ptr->to_address << " ";
            std::cout << ptr->from_address << " ";
            std::cout << ptr->value << " ";
            std::cout << ptr->block_no << " ";
            std::cout << ptr->timestamp << " ";
            std::cout << ptr->contract_address << " " << endl;
        }

        Vec<RawTxDetail> erc721_txs = get_erc721_transfer_history_blocking(
            "0x668f126b87936df4f9a98f18c44eb73868fffea0",
            "0xbd6b9a1A0477d64E99F660b7b7C205f4604E4Ff3",
            QueryOption::ByContract, CRONOSCAN_API_KEY);

        for (Vec<RawTxDetail>::iterator ptr = erc721_txs.begin();
             ptr < erc721_txs.end(); ptr++) {
            std::cout << ptr->hash << " ";
            std::cout << ptr->to_address << " ";
            std::cout << ptr->from_address << " ";
            std::cout << ptr->value << " ";
            std::cout << ptr->block_no << " ";
            std::cout << ptr->timestamp << " ";
            std::cout << ptr->contract_address << " " << endl;
        }
    }

    // Blockscout examples
    Vec<RawTokenResult> tokens_txs =
        get_tokens_blocking("https://blockscout.com/xdai/mainnet/api",
                            "0x652d53227d7013f3FbBeA542443Dc2eeF05719De");
    for (Vec<RawTokenResult>::iterator ptr = tokens_txs.begin();
         ptr < tokens_txs.end(); ptr++) {
        std::cout << ptr->balance << " ";
        std::cout << ptr->contract_address << " ";
        std::cout << ptr->decimals << " ";
        std::cout << ptr->id << " ";
        std::cout << ptr->name << " ";
        std::cout << ptr->symbol << " ";
        std::cout << ptr->token_type << endl;
    }

    Vec<RawTxDetail> token_transfer_txs = get_token_transfers_blocking(
        "https://cronos.org/explorer/testnet3/api",
        "0x841a15D12aEc9c6039FD132c2FbFF112eD355700", "",
        QueryOption::ByAddress);
    for (Vec<RawTxDetail>::iterator ptr = token_transfer_txs.begin();
         ptr < token_transfer_txs.end(); ptr++) {
        std::cout << ptr->hash << " ";
        std::cout << ptr->to_address << " ";
        std::cout << ptr->from_address << " ";
        std::cout << ptr->value << " ";
        std::cout << ptr->block_no << " ";
        std::cout << ptr->timestamp << " ";
        std::cout << ptr->contract_address << " " << endl;
    }

    test_crypto_pay();
}

// convert byte array to hex string
String bytes_to_hex_string(Vec<uint8_t> bytes) {
    stringstream ret;
    ret << std::hex;
    for (int i = 0; i < bytes.size(); i++) {
        ret << std::setw(2) << std::setfill('0') << (int)bytes[i];
    }
    return ret.str();
}

String address_to_hex_string(::std::array<::std::uint8_t, 20> bytes) {
    stringstream ret;
    ret << std::hex;
    for (int i = 0; i < 20; i++) {
        ret << std::setw(2) << std::setfill('0') << (int)bytes[i];
    }
    return ret.str();
}

// if session already exists, restore session
Box<WalletconnectClient> make_new_client(std::string filename) {

    ifstream file(filename.c_str());
    if (file.is_open()) {
        std::string sessioninfostring((istreambuf_iterator<char>(file)),
                                      istreambuf_iterator<char>());
        Box<WalletconnectClient> client =
            walletconnect_restore_client(sessioninfostring);
        return client;
    } else {
        Box<WalletconnectClient> client = walletconnect_new_client(
            "Defi WalletConnect example.", "http://localhost:8080/",
            Vec<rust::String>(), "Defi WalletConnect Web3 Example", 25);
        std::cout << "qrcode= " << client->get_connection_string() << endl;

        return client;
    }
}

/**
@summary    make new client2
@param      filename : which stores session informaiton as json
@return     client : Walletconnect2Client, ownership is transferred to caller
@throws     Error : if sessioninfo is invalid, throw error
*/

Box<Walletconnect2Client> make_new_client2(std::string filename) {

    ifstream file(filename.c_str());
    if (file.is_open()) {
        std::string sessioninfostring((istreambuf_iterator<char>(file)),
                                      istreambuf_iterator<char>());
        Box<Walletconnect2Client> client =
            walletconnect2_restore_client(sessioninfostring);
        return client;
    } else {
        // read env
        String relay_server_string = "wss://relay.walletconnect.com";
        String required_namespaces_json =
            "{\"eip155\":{\"methods\":[\"eth_sendTransaction\",\"eth_"
            "signTransaction\",\"eth_sign\",\"personal_sign\",\"eth_"
            "signTypedData\"],\"chains\":[\"eip155:5\"],\"events\":["
            "\"chainChanged\",\"accountsChanged\"]}}";
        String client_meta_json =
            "{\"description\":\"Defi WalletConnect v2 "
            "example.\",\"url\":\"http://localhost:8080/"
            "\",\"icons\":[],\"name\":\"Defi WalletConnect Web3 Example\"}";

        String projectid = getEnv("NEXT_PUBLIC_PROJECT_ID");
        Box<Walletconnect2Client> client = walletconnect2_client_new(
            relay_server_string.c_str(), projectid.c_str(),
            required_namespaces_json.c_str(), client_meta_json.c_str());
        std::cout << "qrcode= " << client->get_connection_string() << endl;

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
}
void UserWalletConnectCallback::onConnecting(
    const WalletConnectSessionInfo &sessioninfo) const {
    std::cout << "user c++ onConnecting" << std::endl;
    print_session(sessioninfo);
    // this is testing purpose, comment this line for actual test
    exit(0);
}
void UserWalletConnectCallback::onUpdated(
    const WalletConnectSessionInfo &sessioninfo) const {
    std::cout << "user c++ onUpdated" << std::endl;
    print_session(sessioninfo);
}

/**
 * @summary     test wallet connect 1.0    (will be deprecated)
 * @description basic test for wallet connect 1.0
 */
void test_wallet_connect() {
    std::string mycronosrpc = getEnv("CRONOSRPC").c_str();
    bool test_personal = true;
    bool test_basic = false;
    bool test_nft = false;
    std::string filename = "sessioninfo.json";
    try {
        Box<WalletconnectClient> client = make_new_client(filename);
        WalletConnectCallback *usercallbackraw =
            new UserWalletConnectCallback();
        std::unique_ptr<WalletConnectCallback> usercallback(usercallbackraw);
        client->setup_callback_blocking(std::move(usercallback));
        String uri = client->print_uri();
        WalletConnectEnsureSessionResult result =
            client->ensure_session_blocking();

        String sessioninfo = client->save_client();
        {
            ofstream outfile(filename);
            outfile.write(sessioninfo.c_str(), sessioninfo.size());
        }

        assert(result.addresses.size() > 0);

        if (test_personal) {
            /* message signing */
            Vec<uint8_t> sig1 = client->sign_personal_blocking(
                "hello", result.addresses[0].address);
            std::cout << "signature=" << bytes_to_hex_string(sig1).c_str()
                      << std::endl;
            std::cout << "signature length=" << sig1.size() << endl;
        }

        if (test_basic) {
            std::string fromaddress = getenv("MYFROMADDRESS");
            std::cout << "mycronosrpc=" << mycronosrpc << endl;
            std::cout << "fromaddress=" << fromaddress << endl;
            std::string toaddress = getenv("MYTOADDRESS");
            std::cout << "toaddress=" << toaddress << endl;
            std::string mynonce = org::defi_wallet_core::get_eth_nonce(
                                      fromaddress.c_str(), mycronosrpc)
                                      .c_str();
            std::cout << "nonce=" << mynonce << endl;
            WalletConnectTxEip155 info;
            info.to = toaddress;
            info.common.gas_limit = "21000"; // gas limit
            info.common.gas_price = "10000"; // gas price
            info.value = "100000000000000";  // 0.0001 eth
            info.data = Vec<uint8_t>();
            info.common.nonce = mynonce;
            info.common.chainid = 1;

            Vec<uint8_t> rawtx = client->sign_eip155_transaction_blocking(
                info, result.addresses[0].address);

            auto receipt = org::defi_wallet_core::broadcast_eth_signed_raw_tx(
                rawtx, mycronosrpc, 3000);
            std::cout << "transaction_hash="
                      << bytes_to_hex_string(receipt.transaction_hash).c_str()
                      << endl;
        }

        if (test_nft) {
            std::string contractaddress = getenv("MYCONTRACTADDRESS");
            std::string fromaddress = getenv("MYFROMADDRESS");
            std::cout << "mycronosrpc=" << mycronosrpc << endl;
            std::cout << "fromaddress=" << fromaddress << endl;
            std::string toaddress = getenv("MYTOADDRESS");
            std::cout << "toaddress=" << toaddress << endl;
            std::string mynonce = org::defi_wallet_core::get_eth_nonce(
                                      fromaddress.c_str(), mycronosrpc)
                                      .c_str();
            std::cout << "nonce=" << mynonce << endl;
            WalletConnectTxCommon common;
            // TODO use variables
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

            common.nonce = mynonce;
            common.gas_price = "21000";
            common.gas_limit = "100000";
            common.chainid = 1;
            common.web3api_url = mycronosrpc.c_str();

            rust::Vec<uint8_t> rawtx = client->sign_contract_transaction(
                contract_action, common, result.addresses[0].address);

            auto receipt = org::defi_wallet_core::broadcast_eth_signed_raw_tx(
                rawtx, mycronosrpc, 3000);
            std::cout << "transaction_hash="
                      << bytes_to_hex_string(receipt.transaction_hash).c_str()
                      << endl;
        }
    } catch (const cxxbridge1::Error e) {
        std::cout << "wallet connect error=" << e.what() << std::endl;
    }
}

/**
 * @summary     test wallet connect 2.0
 * @description basic test for wallet connect 2.0
 */
void test_wallet_connect2() {
    std::string mycronosrpc = getEnv("CRONOSRPC").c_str();
    bool test_personal = false;
    bool test_basic = true;
    bool test_nft = false;
    std::string filename = "sessioninfo2.json";
    bool exit_program = false;
    try {
        Box<Walletconnect2Client> client = make_new_client2(filename);
        String uri = client->print_uri();
        std::cout << "uri= " << uri.c_str() << std::endl;
        WalletConnect2EnsureSessionResult result =
            client->ensure_session_blocking(60000);
        std::cout << "session result=" << result.eip155.accounts.size()
                  << std::endl;

        std::cout << "ping" << endl;
        String pingresult = client->ping_blocking(60000);
        std::cout << "ping result=" << pingresult.c_str() << std::endl;

        // spawn thread
        std::thread pollingthread([&]() {
            try {
                bool *exitthread = &exit_program;
                while (!(*exitthread)) {
                    try {
                        String ret = client->poll_events_blocking(1000);
                        std::cout << "poll events result=" << ret.c_str()
                                  << std::endl;
                    } catch (const std::exception &e) {
                    }
                }
            } catch (const std::exception &e) {
                std::cout << "wallet connect error=" << e.what() << std::endl;
            }
        });
        String sessioninfo = client->save_client();
        {
            ofstream outfile(filename);
            outfile.write(sessioninfo.c_str(), sessioninfo.size());
        }

        assert(result.eip155.accounts.size() > 0);

        if (test_personal) {
            Vec<uint8_t> sig1 = client->sign_personal_blocking(
                "hello", result.eip155.accounts.at(0).address.address);
            std::cout << "signature length=" << sig1.size() << endl;
        }

        if (test_basic) {
            std::string fromaddress = getenv("MYFROMADDRESS");
            std::cout << "mycronosrpc=" << mycronosrpc << endl;
            std::cout << "fromaddress=" << fromaddress << endl;
            std::string toaddress = getenv("MYTOADDRESS");
            std::cout << "toaddress=" << toaddress << endl;
            std::string mynonce = org::defi_wallet_core::get_eth_nonce(
                                      fromaddress.c_str(), mycronosrpc)
                                      .c_str();
            std::cout << "nonce=" << mynonce << endl;
            WalletConnectTxEip155 info;
            info.to = toaddress;
            info.common.gas_limit = "21000"; // gas limit
            info.common.gas_price = "10000"; // gas price
            info.value = "100000000000000";  // 0.0001 eth
            info.data = Vec<uint8_t>();
            info.common.nonce = mynonce;
            info.common.chainid = 1;

            assert(result.eip155.accounts.size() > 0);
            Vec<uint8_t> rawtx = client->sign_eip155_transaction_blocking(
                info, result.eip155.accounts[0].address.address);
        }

        std::cout << "enter q to exit" << std::endl;
        while (true) {
            // read input, if q is pressed, quit
            char c = getchar();
            if (c == 'q') {
                exit_program = true;
                std::cout << "exit program" << endl;
                break;
            }
        }
        pollingthread.join();

    } catch (const std::exception &e) {
        std::cout << "wallet connect error=" << e.what() << std::endl;
    }
}

// pay api examples
void test_crypto_pay() {
    if (PAY_API_KEY == "")
        return;

    std::atomic<bool> stop_thread_1{false};
    String id = "";
    std::thread t1(websocket_client_thread, std::ref(stop_thread_1),
                   std::ref(id));

    OptionalArguments opiton_args;
    opiton_args.description = "Crypto.com Tee (Unisex)";
    CryptoComPaymentResponse resp =
        create_payment(PAY_API_KEY, "2500", "USD", opiton_args);
    std::cout << "create payment:" << resp.id << " ";
    std::cout << resp.main_app_qr_code << " ";
    std::cout << resp.onchain_deposit_address << " ";
    std::cout << resp.base_amount << " ";
    std::cout << resp.currency << " ";
    std::cout << resp.expiration << " ";
    std::cout << resp.status << endl;

    std::this_thread::sleep_for(std::chrono::milliseconds(3000));
    stop_thread_1 = true; // force stopping websocket thread after timeout
    id = resp.id;         // pass the id to the thread
    t1.join();            // pauses until t1 finishes
}

// A simple websocket client thread
void websocket_client_thread(std::atomic<bool> &stop_thread, String &id) {
    using easywsclient::WebSocket;
    String r_port = PAY_WEBSOCKET_PORT;
    std::string port = r_port.c_str();
    std::unique_ptr<WebSocket> ws(
        WebSocket::from_url("ws://127.0.0.1:" + port));
    if (ws == nullptr)
        return;
    while (ws->getReadyState() != WebSocket::CLOSED) {
        WebSocket::pointer wsp =
            &*ws; // <-- because a unique_ptr cannot be copied into a lambda
        ws->poll();
        ws->dispatch([wsp](std::string msg) {
            // std::cout << "Receive webhook event: " << msg << endl;
            try {
                auto message = json::parse(msg);
                assert(message.at("type") == "payment.created");
                String id = message.at("data").at("object").at("id");
                CryptoComPaymentResponse resp = get_payment(PAY_API_KEY, id);
                std::cout << "get payment: " << resp.id << " ";
                std::cout << resp.main_app_qr_code << " ";
                std::cout << resp.onchain_deposit_address << " ";
                std::cout << resp.base_amount << " ";
                std::cout << resp.currency << " ";
                std::cout << resp.expiration << " ";
                std::cout << resp.status << endl;
                wsp->close();
            } catch (const nlohmann::detail::parse_error &e) {
                std::cout << e.what() << endl;
                wsp->close();
            }
        });
        if (stop_thread) {
            return;
        }
    }
    std::cout << "websocket client thread ends" << endl;
}
