#include "include/extra-cpp-bindings/src/lib.rs.h"
#include "include/rust/cxx.h"
#include <iostream>
using namespace com::crypto::game_sdk;
using namespace rust;
using namespace std;

// Read CronoScan api key in env
const String CRONOSCAN_API_KEY = getenv("CRONOSCAN_API_KEY");

int main(int argc, char *argv[]) {
  // Get a list of "ERC721 - Token Transfer Events" by Address
  // Returns up to a maximum of the last 10000 transactions only
  // https://cronoscan.com/tokentxns-nft?a=0x668f126b87936df4f9a98f18c44eb73868fffea0
  Vec<RawTxDetail> erc721_txs = get_erc721_transfer_history_blocking(
      "0x668f126b87936df4f9a98f18c44eb73868fffea0", "", QueryOption::ByAddress,
      CRONOSCAN_API_KEY);

  for (Vec<RawTxDetail>::iterator ptr = erc721_txs.begin();
       ptr < erc721_txs.end(); ptr++) {
    cout << "hash: " << ptr->hash << " ";
    cout << "to: " << ptr->to_address << " ";
    cout << "from: " << ptr->from_address << " ";
    cout << "TokenID:" << ptr->value << " ";
    cout << "block_no: " << ptr->block_no << " ";
    cout << "timestamp: " << ptr->timestamp << " ";
    cout << "contract: " << ptr->contract_address << " " << endl;
  }

  cout << "A total of " << erc721_txs.size() << " transactions" << endl;

  // Get a list of "ERC721 - Token Transfer Events" ByAddressAndContract
  // Returns up to a maximum of the last 10000 transactions only
  // https://cronoscan.com/token/0x562f021423d75a1636db5be1c4d99bc005ccebfe?a=0x668f126b87936df4f9a98f18c44eb73868fffea0
  erc721_txs = get_erc721_transfer_history_blocking(
      "0x668f126b87936df4f9a98f18c44eb73868fffea0",
      "0x562F021423D75A1636DB5bE1C4D99Bc005ccebFe",
      QueryOption::ByAddressAndContract, CRONOSCAN_API_KEY);

  for (Vec<RawTxDetail>::iterator ptr = erc721_txs.begin();
       ptr < erc721_txs.end(); ptr++) {
    cout << "hash: " << ptr->hash << " ";
    cout << "to: " << ptr->to_address << " ";
    cout << "from: " << ptr->from_address << " ";
    cout << "TokenID:" << ptr->value << " ";
    cout << "block_no: " << ptr->block_no << " ";
    cout << "timestamp: " << ptr->timestamp << " ";
    cout << "contract: " << ptr->contract_address << " " << endl;
  }
  cout << "A total of " << erc721_txs.size() << " transactions" << endl;

  // Get a list of "ERC721 - Token Transfer Events" ByContract
  // Returns up to a maximum of the last 10000 transactions only
  // https://cronoscan.com/token/0x18b73d1f9e2d97057dec3f8d6ea9e30fcadb54d7
  erc721_txs = get_erc721_transfer_history_blocking(
      "", "0x18b73D1f9e2d97057deC3f8D6ea9e30FCADB54D7", QueryOption::ByContract,
      CRONOSCAN_API_KEY);
  for (Vec<RawTxDetail>::iterator ptr = erc721_txs.begin();
       ptr < erc721_txs.end(); ptr++) {
    cout << "hash: " << ptr->hash << " ";
    cout << "to: " << ptr->to_address << " ";
    cout << "from: " << ptr->from_address << " ";
    cout << "TokenID:" << ptr->value << " ";
    cout << "block_no: " << ptr->block_no << " ";
    cout << "timestamp: " << ptr->timestamp << " ";
    cout << "contract: " << ptr->contract_address << " " << endl;
  }

  cout << "A total of " << erc721_txs.size() << " transactions" << endl;
  return 0;
}
