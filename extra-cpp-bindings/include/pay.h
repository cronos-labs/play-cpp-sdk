#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace com {
namespace crypto {
namespace game_sdk {

/// optional arguments for creating a payment on Crypto.com Pay
struct OptionalArguments {
    /// An arbitrary string attached to the object.
    rust::String description;
    /// Set of key-value pairs that you can attach to an object. This can be
    /// useful for storing additional information about the object in a
    /// structured format.
    rust::String metadata;
    /// Merchant provided order ID for this payment.
    rust::String order_id;
    /// The URL for payment page to redirect back to when the payment becomes
    /// succeeded. It is required for redirection flow.
    rust::String return_url;
    /// The URL for payment page to redirect to when the payment is failed or
    /// cancelled.
    rust::String cancel_url;
    /// ID of the sub-merchant associated with this payment. It is required for
    /// merchant acquirers.
    rust::String sub_merchant_id;
    /// Whether to allow the customer to pay by Other Cryptocurrency Wallets for
    /// this payment. If not specified, the setting in merchant account will be
    /// used.
    bool onchain_allowed;
    /// Time at which the payment expires. Measured in seconds since the Unix
    /// epoch. If 0, it will expire after the default period(10 minutes).
    uint64_t expired_at;

    OptionalArguments();
    rust::Str get_description() const;
    rust::Str get_metadata() const;
    rust::Str get_order_id() const;
    rust::Str get_return_url() const;
    rust::Str get_cancel_url() const;
    rust::Str get_sub_merchant_id() const;
    bool get_onchain_allowed() const;
    uint64_t get_expired_at() const;
};

} // namespace game_sdk
} // namespace crypto
} // namespace com
