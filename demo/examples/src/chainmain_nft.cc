#include <cassert>
#include <chrono>
#include <defi-wallet-core-cpp/src/nft.rs.h>
#include <iostream>
#include <rust/cxx.h>
#include <thread>
using namespace org::defi_wallet_core;

CosmosSDKTxInfoRaw build_txinfo() {
  CosmosSDKTxInfoRaw ret;
  ret.account_number = 0;
  ret.sequence_number = 0;
  ret.gas_limit = 5000000;
  ret.fee_amount = 25000000000;
  ret.fee_denom = "basecro";
  ret.timeout_height = 0;
  ret.memo_note = "";
  ret.chain_id = "chainmain-1";
  ret.coin_type = 394;
  ret.bech32hrp = "cro";
  return ret;
}

int main(int argc, char *argv[]) {
  CosmosSDKTxInfoRaw tx_info = build_txinfo();

  rust::String myservertendermint = "http://127.0.0.1:26807";
  rust::String mygrpc = "http://127.0.0.1:26803";
  rust::String myservercosmos = "http://127.0.0.1:26804";

  rust::String from = "cro1u08u5dvtnpmlpdq333uj9tcj75yceggszxpnsy";
  rust::String to = "cro1apdh4yc2lnpephevc6lmpvkyv6s5cjh652n6e4";

  rust::Box<Wallet> signer1_wallet =
      restore_wallet("shed crumble dismiss loyal latin million oblige gesture "
                     "shrug still oxygen custom remove ribbon disorder palace "
                     "addict again blanket sad flock consider obey popular",
                     "");
  rust::Box<PrivateKey> signer1_privatekey =
      signer1_wallet->get_key("m/44'/394'/0'/0/0");

  rust::Box<Wallet> signer2_wallet =
      restore_wallet("night renew tonight dinner shaft scheme domain oppose "
                     "echo summer broccoli agent face guitar surface belt "
                     "veteran siren poem alcohol menu custom crunch index",
                     "");
  rust::Box<PrivateKey> signer2_privatekey =
      signer2_wallet->get_key("m/44'/394'/0'/0/0");

  CosmosAccountInfoRaw detailinfo =
      query_account_details_info(myservercosmos, from);
  auto signer1_sn = detailinfo.sequence_number;
  auto signer1_ac = detailinfo.account_number;

  detailinfo = query_account_details_info(myservercosmos, to);
  auto signer2_sn = detailinfo.sequence_number;
  auto signer2_ac = detailinfo.account_number;

  tx_info.account_number = signer1_ac;
  tx_info.sequence_number = signer1_sn;

  // chainmain nft tests
  auto denom_id = "testdenomid";
  auto denom_name = "testdenomname";
  auto schema = R""""(
  {
    "title": "Asset Metadata",
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "testidentity"
      },
      "description": {
        "type": "string",
        "description": "testdescription"
      },
      "image": {
        "type": "string",
        "description": "testdescription"
      }
    }
  })"""";

  // issue: from
  // signer1_sn += 1; // No need to add sn here, it is the first one
  tx_info.sequence_number = signer1_sn;
  rust::Vec<uint8_t> signedtx = get_nft_issue_denom_signed_tx(
      tx_info, *signer1_privatekey, denom_id, denom_name, schema);

  rust::String resp = broadcast_tx(myservertendermint, signedtx).tx_hash_hex;
  std::cout << "issue response: " << resp << std::endl;

  auto token_id = "testtokenid";
  auto token_name = "testtokenname";
  auto token_uri = "testtokenuri";
  auto token_data = "";

  // mint: from -> to
  signer1_sn += 1;
  tx_info.sequence_number = signer1_sn;
  signedtx =
      get_nft_mint_signed_tx(tx_info, *signer1_privatekey, token_id, denom_id,
                             token_name, token_uri, token_data, to);
  resp = broadcast_tx(myservertendermint, signedtx).tx_hash_hex;
  std::cout << "mint response: " << resp << std::endl;

  std::this_thread::sleep_for(std::chrono::seconds(3));
  rust::Box<GrpcClient> grpc_client = new_grpc_client(mygrpc);

  Pagination pagination;
  assert(pagination.enable == false);
  assert(pagination.key.size() == 0);
  assert(pagination.offset == 0);
  assert(pagination.limit == 100);
  assert(pagination.count_total == false);
  assert(pagination.reverse == false);
  rust::Vec<Denom> denoms = grpc_client->denoms(pagination);
  assert(denoms.size() == 1);
  assert(denoms[0].id == denom_id);
  assert(denoms[0].name == denom_name);
  assert(denoms[0].schema == schema);
  assert(denoms[0].creator == from);

  BaseNft nft = grpc_client->nft(denom_id, token_id);
  std::cout << "nft: " << nft.to_string() << std::endl;
  assert(nft.id == token_id);
  assert(nft.name == token_name);
  assert(nft.uri == token_uri);
  assert(nft.data == token_data);
  assert(nft.owner == to);

  Collection collection = grpc_client->collection(denom_id, pagination);
  std::cout << "collection: " << collection.to_string() << std::endl;
  Owner owner = grpc_client->owner(denom_id, to, pagination);
  std::cout << "owner: " << owner.to_string() << std::endl;
  assert(owner.address == to);
  assert(owner.id_collections.size() == 1);
  assert(owner.id_collections[0].denom_id == denom_id);
  assert(owner.id_collections[0].token_ids.size() == 1);
  assert(owner.id_collections[0].token_ids[0] == token_id);

  // transfer: to -> from
  tx_info.account_number = signer2_ac;
  tx_info.sequence_number = signer2_sn;
  signedtx = get_nft_transfer_signed_tx(tx_info, *signer2_privatekey, token_id,
                                        denom_id, from);
  resp = broadcast_tx(myservertendermint, signedtx).tx_hash_hex;
  std::cout << "transfer response: " << resp << std::endl;
  std::this_thread::sleep_for(std::chrono::seconds(3));
  nft = grpc_client->nft(denom_id, token_id);
  std::cout << "nft: " << nft.to_string() << std::endl;
  assert(nft.id == token_id);
  assert(nft.name == token_name);
  assert(nft.uri == token_uri);
  assert(nft.data == token_data);
  assert(nft.owner == from);
  owner = grpc_client->owner(denom_id, from, pagination);
  std::cout << "owner: " << owner.to_string() << std::endl;
  assert(owner.address == from);
  assert(owner.id_collections.size() == 1);
  assert(owner.id_collections[0].denom_id == denom_id);
  assert(owner.id_collections[0].token_ids.size() == 1);
  assert(owner.id_collections[0].token_ids[0] == token_id);

  // edit
  tx_info.account_number = signer1_ac;
  signer1_sn += 1;
  tx_info.sequence_number = signer1_sn;
  signedtx = get_nft_edit_signed_tx(tx_info, *signer1_privatekey, token_id,
                                    denom_id, "newname", "newuri", "newdata");
  resp = broadcast_tx(myservertendermint, signedtx).tx_hash_hex;
  std::cout << "edit response: " << resp << std::endl;
  std::this_thread::sleep_for(std::chrono::seconds(3));
  nft = grpc_client->nft(denom_id, token_id);
  std::cout << "nft: " << nft.to_string() << std::endl;
  assert(nft.id == token_id);
  assert(nft.name == "newname");
  assert(nft.uri == "newuri");
  assert(nft.data == "newdata");
  assert(nft.owner == from);
  uint64_t supply = grpc_client->supply(denom_id, from);
  std::cout << "supply: " << supply << std::endl;
  assert(supply == 1);

  // burn
  signer1_sn += 1;
  tx_info.sequence_number = signer1_sn;
  signedtx =
      get_nft_burn_signed_tx(tx_info, *signer1_privatekey, token_id, denom_id);
  resp = broadcast_tx(myservertendermint, signedtx).tx_hash_hex;
  std::cout << "burn response: " << resp << std::endl;
  std::this_thread::sleep_for(std::chrono::seconds(3));
  supply = grpc_client->supply(denom_id, from);
  std::cout << "supply: " << supply << std::endl;
  assert(supply == 0);

  return 0;
}
