#include <extra-cpp-bindings/src/lib.rs.h>
#include <iostream>
#include <rust/cxx.h>
using namespace com::crypto::game_sdk;

inline rust::String getEnv(rust::String key) {
  rust::String ret;
  if (getenv(key.c_str()) != nullptr) {
    ret = getenv(key.c_str());
  }
  return ret;
}

// Read CronoScan api key in env
const rust::String CRONOSCAN_API_KEY = getEnv("CRONOSCAN_API_KEY");

int main(int argc, char *argv[]) {
  if (CRONOSCAN_API_KEY == "")
    return -1;

  // Get a list of "CRC20 - Token Transfer Events" by Address
  // Returns up to a maximum of the last 10000 transactions only
  // https://cronoscan.com/tokentxns?a=0xa9b34a4b568e640d5e5d1e6e13101025e1262864
  rust::Vec<RawTxDetail> erc20_txs = get_erc20_transfer_history_blocking(
      "0xa9b34a4b568e640d5e5d1e6e13101025e1262864", "", QueryOption::ByAddress,
      CRONOSCAN_API_KEY);

  for (rust::Vec<RawTxDetail>::iterator ptr = erc20_txs.begin();
       ptr < erc20_txs.end(); ptr++) {
    std::cout << "hash: " << ptr->hash << " ";
    std::cout << "to: " << ptr->to_address << " ";
    std::cout << "from: " << ptr->from_address << " ";
    std::cout << "value:" << ptr->value << " ";
    std::cout << "block_no: " << ptr->block_no << " ";
    std::cout << "timestamp: " << ptr->timestamp << " ";
    std::cout << "contract: " << ptr->contract_address << " " << std::endl;
  }

  std::cout << "A total of " << erc20_txs.size() << " transactions"
            << std::endl;

  // Get a list of "CRC20 - Token Transfer Events" by ByAddressAndContract
  // Returns up to a maximum of the last 10000 transactions only
  // https://cronoscan.com/token/0x2d03bece6747adc00e1a131bba1469c15fd11e03?a=0xa9b34a4b568e640d5e5d1e6e13101025e1262864
  erc20_txs = get_erc20_transfer_history_blocking(
      "0xa9b34a4b568e640d5e5d1e6e13101025e1262864",
      "0x2D03bECE6747ADC00E1a131BBA1469C15fD11e03",
      QueryOption::ByAddressAndContract, CRONOSCAN_API_KEY);

  for (rust::Vec<RawTxDetail>::iterator ptr = erc20_txs.begin();
       ptr < erc20_txs.end(); ptr++) {
    std::cout << "hash: " << ptr->hash << " ";
    std::cout << "to: " << ptr->to_address << " ";
    std::cout << "from: " << ptr->from_address << " ";
    std::cout << "value:" << ptr->value << " ";
    std::cout << "block_no: " << ptr->block_no << " ";
    std::cout << "timestamp: " << ptr->timestamp << " ";
    std::cout << "contract: " << ptr->contract_address << " " << std::endl;
  }
  std::cout << "A total of " << erc20_txs.size() << " transactions"
            << std::endl;

  // Get a list of "CRC20 - Token Transfer Events" by ByContract
  // Returns up to a maximum of the last 10000 transactions only
  erc20_txs = get_erc20_transfer_history_blocking(
      "", "0x66e428c3f67a68878562e79A0234c1F83c208770", QueryOption::ByContract,
      CRONOSCAN_API_KEY);

  std::cout << "A total of " << erc20_txs.size() << " transactions"
            << std::endl;
  return 0;
}
