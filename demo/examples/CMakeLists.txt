# add examples
include_directories(../sdk/include/)
include_directories(../third_party/)

add_executable(new_wallet src/new_wallet.cc)
target_link_libraries(new_wallet PUBLIC play_cpp_sdk)

add_executable(restore_wallet src/restore_wallet.cc)
target_link_libraries(restore_wallet PUBLIC play_cpp_sdk)

add_executable(chainmain_bank_send src/chainmain_bank_send.cc)
target_link_libraries(chainmain_bank_send PUBLIC play_cpp_sdk)

add_executable(chainmain_nft src/chainmain_nft.cc)
target_link_libraries(chainmain_nft PUBLIC play_cpp_sdk)

add_executable(uint src/uint.cc)
target_link_libraries(uint PUBLIC play_cpp_sdk)

add_executable(eth src/eth.cc)
target_link_libraries(eth PUBLIC play_cpp_sdk)

add_executable(eth_login src/eth_login.cc)
target_link_libraries(eth_login PUBLIC play_cpp_sdk)

add_executable(erc20 src/erc20.cc)
target_link_libraries(erc20 PUBLIC play_cpp_sdk)

add_executable(erc721 src/erc721.cc)
target_link_libraries(erc721 PUBLIC play_cpp_sdk)

add_executable(erc1155 src/erc1155.cc)
target_link_libraries(erc1155 PUBLIC play_cpp_sdk)

add_executable(get_erc20_transfer_history_blocking src/get_erc20_transfer_history_blocking.cc)
target_link_libraries(get_erc20_transfer_history_blocking PUBLIC play_cpp_sdk)

add_executable(get_erc721_transfer_history_blocking src/get_erc721_transfer_history_blocking.cc)
target_link_libraries(get_erc721_transfer_history_blocking PUBLIC play_cpp_sdk)

add_executable(get_tokens_blocking src/get_tokens_blocking.cc)
target_link_libraries(get_tokens_blocking PUBLIC play_cpp_sdk)

add_executable(get_token_transfers_blocking src/get_token_transfers_blocking.cc)
target_link_libraries(get_token_transfers_blocking PUBLIC play_cpp_sdk)

add_executable(create_payment src/create_payment.cc ../third_party/easywsclient/easywsclient.cpp)
target_link_libraries(create_payment PUBLIC play_cpp_sdk)

add_executable(wallet_connect src/wallet_connect.cc)
target_link_libraries(wallet_connect PUBLIC play_cpp_sdk)
