use std::sync::Arc;

use breez_sdk_liquid::ReceivePaymentRequest;
use rocket::serde::json::{serde_json::json, Value};
use rocket::{get, State};

use crate::*;

pub(crate) fn get_ln_address(config: &State<Arc<AppConfig>>, network: ChosenNetwork) -> String {
    format!("{network:?}@{}", &config.domain).to_lowercase()
}

pub(crate) fn get_lnurl_pay_qr(config: &State<Arc<AppConfig>>, network: ChosenNetwork) -> String {
    let ln_address = get_ln_address(config, network);
    QrCode::with_error_correction_level(ln_address, EcLevel::L)
        .unwrap()
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}

#[get("/.well-known/lnurlp/<user>")]
pub(crate) fn get_lnurl_params(config: &State<Arc<AppConfig>>, user: &str) -> Value {
    let callback = format!("https://{}/{user}/invoice", config.domain);
    build_pay_response_ok(
        &callback,
        config.min_sendable_msat,
        config.max_sendable_msat,
    )
}

#[get("/<network>/invoice?<amount>")]
pub(crate) async fn get_invoice(
    ls_sdk_mainnet: &State<MainnetWallet>,
    ls_sdk_testnet: &State<TestnetWallet>,
    network: ChosenNetwork,
    amount: u64,
) -> Result<Value, Value> {
    let ls_sdk = match network {
        ChosenNetwork::Mainnet => &ls_sdk_mainnet.0,
        ChosenNetwork::Testnet => &ls_sdk_testnet.0,
    };

    // TODO Validate amount, comment length
    ls_sdk
        .receive_payment(ReceivePaymentRequest {
            invoice_amount_sat: Some(amount / 1_000),
            onchain_amount_sat: None,
        })
        .map(|res| json!({"pr": res.invoice, "routes": [] }))
        .map_err(|err| build_lnurl_response_err(&format!("Failed to get invoice: {err}")))
}

// https://github.com/lnurl/luds/blob/luds/06.md#wallet-to-service-interaction-flow
fn build_pay_response_ok(callback: &str, min_sendable_msat: u64, max_sendable_msat: u64) -> Value {
    // Key and value must have escaped quotes
    let metadata = json!([["text/plain", "tbd-test-metadata"]]).to_string();
    json!({
        "callback": callback,
        "maxSendable": max_sendable_msat,
        "minSendable": min_sendable_msat,
        "metadata": metadata,
        "tag": "payRequest"
    })
}
