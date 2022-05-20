#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace com {
namespace crypto {
namespace game_sdk {

using namespace rust;

/// optional arguments for creating a payment on Crypto.com Pay
struct OptionalArguments {
  /// An arbitrary string attached to the object.
  String description;
  /// Set of key-value pairs that you can attach to an object. This can be useful for storing
  /// additional information about the object in a structured format.
  String metadata;
  /// Merchant provided order ID for this payment.
  String order_id;
  /// The URL for payment page to redirect back to when the payment becomes succeeded. It is
  /// required for redirection flow.
  String return_url;
  /// The URL for payment page to redirect to when the payment is failed or cancelled.
  String cancel_url;
  /// ID of the sub-merchant associated with this payment. It is required for merchant
  /// acquirers.
  String sub_merchant_id;
  /// Whether to allow the customer to pay by Other Cryptocurrency Wallets for this payment. If
  /// not specified, the setting in merchant account will be used.
  bool onchain_allowed;
  /// Time at which the payment expires. Measured in seconds since the Unix epoch. If not
  /// specified, it will expire after the default period(10 minutes).
  uint64_t expired_at;

  OptionalArguments();
  Str get_description() const;
  Str get_metadata() const;
  Str get_order_id() const;
  Str get_return_url() const;
  Str get_cancel_url() const;
  Str get_sub_merchant_id() const;
  bool get_onchain_allowed() const;
  uint64_t get_expired_at() const;
};

} // namespace game_sdk
} // namespace crypto
} // namespace com
