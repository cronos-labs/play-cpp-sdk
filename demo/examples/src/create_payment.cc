#include <atomic>
#include <easywsclient/easywsclient.hpp>
#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <json/single_include/nlohmann/json.hpp>
#include <rust/cxx.h>
#include <thread>
using namespace com::crypto::game_sdk;
using namespace nlohmann;

void websocket_client_thread(std::atomic<bool> &stop_thread, rust::String &id);

inline rust::String getEnv(rust::String key) {
    rust::String ret;
    if (getenv(key.c_str()) != nullptr) {
        ret = getenv(key.c_str());
    }
    return ret;
}

// Read pay api key in env
const rust::String PAY_API_KEY = getEnv("PAY_API_KEY");
// Read websocket port in env
const rust::String PAY_WEBSOCKET_PORT = getEnv("PAY_WEBSOCKET_PORT");

int main(int argc, char *argv[]) {
    if (PAY_API_KEY == "")
        return -1;

    std::atomic<bool> stop_thread_1{false};
    rust::String id = "";
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
    std::cout << resp.status << std::endl;

    std::this_thread::sleep_for(std::chrono::milliseconds(3000));
    stop_thread_1 = true; // force stopping websocket thread after timeout
    id = resp.id;         // pass the id to the thread
    t1.join();            // pauses until t1 finishes

    return 0;
}

// A simple websocket client thread
void websocket_client_thread(std::atomic<bool> &stop_thread, rust::String &id) {
    using easywsclient::WebSocket;
    rust::String r_port = PAY_WEBSOCKET_PORT;
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
            // std::cout << "Receive webhook event: " << msg << std::endl;
            try {
                auto message = json::parse(msg);
                assert(message.at("type") == "payment.created");
                rust::String id = message.at("data").at("object").at("id");
                CryptoComPaymentResponse resp = get_payment(PAY_API_KEY, id);
                std::cout << "get payment: " << resp.id << " ";
                std::cout << resp.main_app_qr_code << " ";
                std::cout << resp.onchain_deposit_address << " ";
                std::cout << resp.base_amount << " ";
                std::cout << resp.currency << " ";
                std::cout << resp.expiration << " ";
                std::cout << resp.status << std::endl;
                wsp->close();
            } catch (const nlohmann::detail::parse_error &e) {
                std::cout << e.what() << std::endl;
                wsp->close();
            }
        });
        if (stop_thread) {
            return;
        }
    }
    std::cout << "websocket client thread ends" << std::endl;
}
