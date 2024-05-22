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
using namespace org::defi_wallet_core;
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
rust::Box<Walletconnect2Client> make_new_client(std::string filename) {

    std::ifstream file(filename.c_str());
    if (file.is_open()) {
        std::string sessioninfostring((std::istreambuf_iterator<char>(file)),
                                      std::istreambuf_iterator<char>());
        rust::Box<Walletconnect2Client> client =
            walletconnect2_restore_client(sessioninfostring);
        return client;
    } else {
        std::string projectid = std::getenv("NEXT_PUBLIC_PROJECT_ID")
                                    ? std::getenv("NEXT_PUBLIC_PROJECT_ID")
                                    : "";
        // assert projectid not ""
        assert(projectid != "");

        rust::Box<Walletconnect2Client> client = walletconnect2_client_new(
            "wss://relay.walletconnect.org", projectid,
            "{\"eip155\":{\"methods\":[\"eth_sendTransaction\",\"eth_"
            "signTransaction\",\"eth_sign\",\"personal_sign\",\"eth_"
            "signTypedData\"],\"chains\":[\"eip155:338\"],\"events\":["
            "\"chainChanged\",\"accountsChanged\"]}}",
            "{\"description\":\"Defi WalletConnect v2 "
            "example.\",\"url\":\"http://localhost:8080/"
            "\",\"icons\":[],\"name\":\"Defi WalletConnect Web3 Example\"}");
        std::cout << "qrcode= " << client->get_connection_string() << std::endl;

        return client;
    }
}
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

int main(int argc, char *argv[]) {
    std::string filename = "sessioninfo.json";
    try {
        rust::Box<Walletconnect2Client> client = make_new_client(filename);

        // Print the QR code on terminal
        rust::String uri = client->print_uri();

        // program is blocked here for waiting connecting
        WalletConnect2EnsureSessionResult result =
            client->ensure_session_blocking(60000);

        // once connected, program continues
        assert(result.eip155.accounts.size() > 0);

        // get the connected session info as string and save it into a file
        rust::String sessioninfo = client->save_client();
        std::cout << "sessioninfo = " << sessioninfo << std::endl;
        std::ofstream outfile(filename);
        outfile.write(sessioninfo.c_str(), sessioninfo.size());
        // it is important to close file and release the session file
        outfile.close();

        bool test_personal = true;
        bool test_basic = false;
        bool test_erc20 = false;

        // sign personal message
        if (test_personal) {
            /* message signing */
            ::std::uint64_t testchainid = result.eip155.accounts[0].chain_id;
            ::std::array<::std::uint8_t, 20> testaddress =
                result.eip155.accounts[0].address.address;
            std::cout << "chainid=" << testchainid << std::endl;
            std::cout << "address="
                      << address_to_hex_string(testaddress).c_str()
                      << std::endl;
            rust::Vec<uint8_t> sig1 =
                client->sign_personal_blocking("hello", testaddress);
            std::cout << "signature=" << bytes_to_hex_string(sig1).c_str()
                      << std::endl;
            std::cout << "signature length=" << sig1.size() << std::endl;
            bool verifyresult =
                client->verify_personal_blocking("hello", sig1, testaddress);
            std::cout << "verify result=" << verifyresult << std::endl;
        }

        // send transaction
        if (test_basic) {
            rust::String block_number =
                get_block_number_blocking("https://evm-t3.cronos.org");
            std::cout << "block number=" << block_number.c_str() << std::endl;

            std::cout << "Get transaction_receipt..." << std::endl;
            rust::String tx_receipt = get_eth_transaction_receipt_blocking(
                "d6dcb26d14f27ce8ae9b394fdecf02d48f5f6f7aea9a159fc0a8114c"
                "26efe2ef",
                "https://evm-t3.cronos.org");

            std::cout << "transaction_receipt=" << tx_receipt.c_str()
                      << std::endl;

            WalletConnectTxEip155 info;
            // send to the connected wallet itself
            // To send to other wallet address, simply
            // info.to = "0x....";
            info.to = rust::String(
                std::string("0x") +
                address_to_hex_string(result.eip155.accounts[0].address.address)
                    .c_str());
            info.value = "1000000000000000000"; // 1 TCRO
            info.common.chainid = result.eip155.accounts[0].chain_id;
            rust::Vec<uint8_t> tx_hash =
                client->send_eip155_transaction_blocking(
                    info, result.eip155.accounts[0].address.address);

            std::cout << "transaction_hash="
                      << bytes_to_hex_string(tx_hash).c_str() << std::endl;

            tx_receipt = get_eth_transaction_receipt_blocking(
                tx_hash, "https://evm-t3.cronos.org");
            std::cout << "transaction_receipt=" << tx_receipt.c_str()
                      << std::endl;

            tx_receipt = wait_for_transaction_receipt_blocking(
                tx_hash, "https://evm-t3.cronos.org");
            std::cout << "transaction_receipt=" << tx_receipt.c_str()
                      << std::endl;

            block_number =
                get_block_number_blocking("https://evm-t3.cronos.org");
            std::cout << "block number=" << block_number.c_str() << std::endl;
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
                address_to_hex_string(result.eip155.accounts[0].address.address)
                    .c_str());
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

            common.chainid = result.eip155.accounts[0].chain_id;
            common.web3api_url =
                "https://evm-dev-t3.cronos.org"; // TODO unnessary for
                                                 // walletconnect

            rust::Vec<uint8_t> tx_hash = client->send_contract_transaction(
                contract_action, common,
                result.eip155.accounts[0].address.address);

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
