#include "extra-cpp-bindings/include/pay.h"
#include "extra-cpp-bindings/src/lib.rs.h"

#include <string>
#include <vector>

namespace com {
namespace crypto {
namespace game_sdk {

OptionalArguments::OptionalArguments() : onchain_allowed{true}, expired_at{0} {}

rust::Str OptionalArguments::get_description() const { return description; }
rust::Str OptionalArguments::get_metadata() const { return metadata; }
rust::Str OptionalArguments::get_order_id() const { return order_id; }
rust::Str OptionalArguments::get_return_url() const { return return_url; }
rust::Str OptionalArguments::get_cancel_url() const { return cancel_url; }
rust::Str OptionalArguments::get_sub_merchant_id() const { return sub_merchant_id; }
bool OptionalArguments::get_onchain_allowed() const { return onchain_allowed; }
uint64_t OptionalArguments::get_expired_at() const { return expired_at; }


} // namespace game_sdk
} // namespace crypto
} // namespace com
