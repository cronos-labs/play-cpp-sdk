use super::error::GameSdkError;
use super::ffi::OptionalArguments;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ResponseData {
    Success(Box<CryptoPayObject>),
    Error { error: CryptoPayErrorObject },
}

// workaround, enable dead_code to suppress the  warnnings
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub(crate) struct CryptoPayErrorObject {
    #[serde(rename = "type")]
    error_type: String,
    code: String,
    error_message: Option<String>,
    param: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CryptoPayObject {
    /// uuid
    pub id: String,
    /// arbitrary_precision amounts
    original_amount: Option<serde_json::Number>,
    pub amount: serde_json::Number,
    amount_refunded: serde_json::Number,
    /// timestamp Measured in seconds since the Unix epoch.
    created: u64,
    /// e.g. "0.01"
    cashback_rate: Option<String>,
    crypto_currency: String,
    /// e.g. "100.0"
    crypto_amount: String,
    /// three-letter currency code: https://pay-docs.crypto.com/#api-reference-resources-payments-pricing-currencies
    pub currency: String,
    /// UUID
    customer_id: Option<String>,
    /// ```ignore
    /// #[derive(Serialize, Deserialize)]
    /// pub struct CustomerProvidedInfo {
    /// email: String,
    /// }```
    customer_provided_info: Option<HashMap<String, serde_json::Value>>,
    data_url: Option<String>,
    payment_url: String,
    return_url: String,
    cancel_url: String,
    description: Option<String>,
    live_mode: bool,
    /// for example {"customer_name": "..."}
    /// ```ignore
    /// #[derive(Serialize, Deserialize)]
    /// pub struct Metadata {
    ///   customer_name: String,
    /// }```
    metadata: Option<HashMap<String, serde_json::Value>>,
    /// uuid
    order_id: Option<String>,
    recipient: String,
    refunded: bool,
    pub status: String,
    /// 10 minutes -- 600 secs
    time_window: Option<u64>,
    /// e.g. 0:00
    remaining_time: Option<String>,
    /// ?
    resource_type: Option<serde_json::Value>,
    resource_id: Option<serde_json::Value>,
    resource: Option<serde_json::Value>,
    merchant_avatar_url: Option<String>,
    /// timestamp Measured in seconds since the Unix epoch.
    settled_at: Option<u64>,
    expired: Option<bool>,
    /// duplicate?
    enable_onchain_payment: Option<bool>,
    onchain_enabled: Option<bool>,
    pub deposit_address: Option<String>,
    current_inbound_fund: Option<CurrentInboundFund>,
    refresh_disabled: Option<bool>,
    ncw_connections: Option<Vec<String>>,
    network_cost: Option<String>,
    defi_swap_transaction: Option<serde_json::Value>,
    /// ```ignore
    /// #[derive(Serialize, Deserialize)]
    /// pub struct SubDepositAddresses {
    ///     #[serde(rename = "BTC")]
    ///     btc: String,
    ///     #[serde(rename = "ETH")]
    ///     eth: String,
    ///     #[serde(rename = "CRO_NATIVE")]
    ///     cro_native: String,
    /// }```
    sub_deposit_addresses: Option<HashMap<String, String>>,
    captured: Option<bool>,
    /// e.g. "External Wallets"
    payment_source: Option<String>,
    /// e.g. "checkout",
    payment_type: Option<String>,
    /// base64 encoded json to be displayed in a QR code
    /// that can be scanned by the main app
    /// (it contains the payment id)
    pub qr_code: Option<String>,
    experimental_features: Option<Vec<String>>,
    sub_merchant_id: Option<serde_json::Value>,
    /// timestamp Measured in seconds since the Unix epoch.
    pub expired_at: Option<u64>,
    pay_later_installments: Option<Vec<PayLaterInstallment>>,
    pay_later_qr_code: Option<String>,
    pay_method: Option<String>,
    allow_pay_later: Option<bool>,
    amount_in_usd: Option<String>,
    amount_in_usdc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CurrentInboundFund {
    id: String,
    txn_id: String,
    deposit_address: String,
    inbound_fund_request_id: String,
    currency: String,
    amount: String,
    status: String,
    confirmed: bool,
    mismatched_reason: Option<String>,
    created_at: String,
    txn_created_at: String,
    from_address: String,
    rating: String,
    tx_index: String,
    block_number: u64,
    updated_at: String,
    meta: HashMap<String, serde_json::Value>,
    payer_id: String,
    rebound_ids: Vec<String>,
    is_within_threshold: bool,
    coin_from: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PayLaterInstallment {
    repay_date: String,
    repay_amount: String,
    currency: String,
}

pub(crate) fn create_payment(
    secret_or_publishable_api_key: &str,
    base_unit_amount: &str,
    currency: &str,
    optional_args: &OptionalArguments,
) -> Result<CryptoPayObject, GameSdkError> {
    const URL: &str = "https://pay.crypto.com/api/payments";
    let mut data = vec![("amount", base_unit_amount), ("currency", currency)];

    let description = optional_args.get_description();
    if !description.is_empty() {
        data.push(("description", description));
    }

    let metadata = optional_args.get_metadata();
    if !metadata.is_empty() {
        data.push(("metadata", metadata));
    }

    let order_id = optional_args.get_order_id();
    if !order_id.is_empty() {
        data.push(("order_id", order_id));
    }

    let return_url = optional_args.get_return_url();
    if !return_url.is_empty() {
        data.push(("return_url", return_url));
    }

    let cancel_url = optional_args.get_cancel_url();
    if !cancel_url.is_empty() {
        data.push(("cancel_url", cancel_url));
    }

    let sub_merchant_id = optional_args.get_sub_merchant_id();
    if !sub_merchant_id.is_empty() {
        data.push(("sub_merchant_id", sub_merchant_id));
    }

    if optional_args.get_onchain_allowed() {
        data.push(("onchain_allowed", "true"));
    } else {
        data.push(("onchain_allowed", "false"));
    }

    let expired_at = optional_args.get_expired_at().to_string();
    if expired_at != "0" {
        data.push(("expired_at", &expired_at));
    }

    let client = reqwest::blocking::Client::new();
    let resp: ResponseData = client
        .post(URL)
        .basic_auth(secret_or_publishable_api_key, Some(""))
        .form(&data)
        .send()?
        .json()?;

    match resp {
        ResponseData::Error { error: err } => Err(GameSdkError::CryptoPayError(err)),
        ResponseData::Success(resp) => Ok(*resp),
    }
}

pub(crate) fn get_payment(
    secret_or_publishable_api_key: &str,
    payment_id: &str,
) -> Result<CryptoPayObject, GameSdkError> {
    let url: String = format!("https://pay.crypto.com/api/payments/{payment_id}");
    let client = reqwest::blocking::Client::new();
    let resp: ResponseData = client
        .get(url)
        .basic_auth(secret_or_publishable_api_key, Some(""))
        .send()?
        .json()?;

    match resp {
        ResponseData::Error { error: err } => Err(GameSdkError::CryptoPayError(err)),
        ResponseData::Success(resp) => Ok(*resp),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_parse_payment_object() {
        let sample = r#"{"id":"a8608fef-05bf-4a43-9d16-6c0f2235ece7",
        "original_amount":100000000000000000000,
        "amount":100000000000000000000,
        "amount_refunded":100000000000000000000,
        "created":1646669903,
        "cashback_rate":"0.01","crypto_currency":"CRO","crypto_amount":"100.0","currency":"CRO",
        "customer_id":null,"customer_provided_info":{"email":"...@gmail.com"},
        "data_url":"https://pay.crypto.com/sdk/payments/a8608fef-05bf-4a43-9d16-6c0f2235ece7?signature=17f717d10617",
        "payment_url":"https://js.crypto.com/sdk/payments/checkout/set_wallet?publishableKey=...",
        "return_url":"http://google.com","cancel_url":"","description":null,
        "live_mode":true,"metadata":null,"order_id":null,
        "recipient":"Thomas CRO Test","refunded":true,"status":"succeeded",
        "time_window":600,"remaining_time":"0:00",
        "resource_type":null,"resource_id":null,"resource":null,
        "merchant_avatar_url":"","settled_at":1646670287,"expired":false,
        "enable_onchain_payment":true,"onchain_enabled":true,
        "deposit_address":"0xD3B8bD9855FFC40cC9E8a8Ea322BAB1bE481565d",
        "current_inbound_fund":{"id":"02d7745b-f644-4522-b277-644cd83b2b42",
        "txn_id":"0x4fdf3539734d32aa23901e254fda5a86c26941c2a8b61f7f9730970d86e07d6f",
        "deposit_address":"0xD3B8bD9855FFC40cC9E8a8Ea322BAB1bE481565d",
        "inbound_fund_request_id":"1724589b-39be-479e-bf5e-e03d0f79c37a",
        "currency":"CRO_CRONOS","amount":"100.0","status":"captured",
        "confirmed":true,"mismatched_reason":null,"created_at":"2022-03-07T16:23:13.453Z",
        "txn_created_at":"2022-03-07T16:18:54.000Z","from_address":"0x0C403e0D57Eeb8f091bCfA8EFdF01c00509f04f2",
        "rating":"unknown","tx_index":"0","block_number":0,"updated_at":"2022-03-07T16:24:47.577Z",
        "meta":{},"payer_id":"f93c5cae753b0cf2a7322f8a30e56ced72c0e07e","rebound_ids":[],
        "is_within_threshold":false,"coin_from":"Cronos"},"refresh_disabled":false,
        "ncw_connections":["ncw_metamask_plugin","ncw_wallet_connect","defi_swap","cro_chain"],
        "network_cost":"45.0","defi_swap_transaction":null,
        "sub_deposit_addresses":{"BTC":"3ET7BqDbKJq2PRWdHqmYBQqwiEZ1qvwvES",
        "ETH":"0xD3B8bD9855FFC40cC9E8a8Ea322BAB1bE481565d",
        "CRO_NATIVE":"cro1w2kvwrzp23aq54n3amwav4yy4a9ahq2kz2wtmj?memo=294567596"},
        "captured":true,"payment_source":"External Wallets","payment_type":"checkout",
        "qr_code":"eyJ0eXBlIjoicGF5bWVudCIsImlkIjoiYTg2MDhmZWYtMDViZi00YTQzLTlkMTYtNmMwZjIyMzVlY2U3In0=",
        "experimental_features":["exptl_onchain_rebound","exptl_cronos","exptl_qr_code_payment",
        "exptl_subscription","exptl_qr_code_refund","exptl_qr_code_rebound"],
        "sub_merchant_id":null,"expired_at":null,"pay_later_installments":
        [{"repay_date":"2022-04-11","repay_amount":"0.0","currency":"USDC"},
        {"repay_date":"2022-04-25","repay_amount":"0.0","currency":"USDC"},{"repay_date":"2022-05-09",
        "repay_amount":"0.0","currency":"USDC"},{"repay_date":"2022-05-23","repay_amount":"0.0","currency":"USDC"}],
        "pay_later_qr_code":"eyJ0eXBlIjoicGF5bWVudCIsImlkIjoiYTg2MDhmZWYtMDViZi00YTQzLTlkMTYtNmMwZjIyMzVlY2U3IiwicGF5X21ldGhvZCI6InBheV9sYXRlciJ9",
        "pay_method":"Other Wallets","allow_pay_later":false,"amount_in_usd":"0.0","amount_in_usdc":"0.0"}"#;
        let _po: CryptoPayObject = serde_json::from_str(sample).expect("parse");
    }
}
