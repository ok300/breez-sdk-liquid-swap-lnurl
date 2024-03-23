use std::sync::Arc;
use std::time::SystemTime;

use rocket::{get, State};
use rocket::serde::json::{serde_json::json, Value};

use crate::*;

// https://github.com/lnurl/luds/blob/luds/17.md
pub(crate) fn get_lnurl_withdraw_link(
    config: &State<Arc<AppConfig>>,
    network: ChosenNetwork,
) -> String {
    format!("https://{}/.well-known/lnurlw/{network:?}", &config.domain).to_lowercase()
}

pub(crate) fn get_lnurl_withdraw_qr(
    config: &State<Arc<AppConfig>>,
    network: ChosenNetwork,
) -> String {
    let lnurl_withdraw_link = get_lnurl_withdraw_link(config, network);
    QrCode::with_error_correction_level(lnurl_withdraw_link, EcLevel::L)
        .unwrap()
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build()
}

#[get("/.well-known/lnurlw/<user>")]
pub(crate) fn get_lnurl_withdraw_params(config: &State<Arc<AppConfig>>, user: &str) -> Value {
    let callback = format!("https://{}/{user}/withdraw", config.domain);

    // TODO Configurable min / max withdrawable
    build_withdraw_response_ok(
        &callback,
        5_000_000,
        100_000_000,
        // config.min_sendable_msat,
        // config.max_sendable_msat,
    )
}

// https://github.com/lnurl/luds/blob/luds/03.md#wallet-to-service-interaction-flow
fn build_withdraw_response_ok(
    callback: &str,
    min_withdrawable_msat: u64,
    max_withdrawable_msat: u64,
) -> Value {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(ts) => json!({
            "tag": "withdrawRequest",
            "callback": callback,
            "k1": format!("{}", ts.as_millis()),
            "defaultDescription": "tbd-test-withdraw-description",
            "minWithdrawable": min_withdrawable_msat,
            "maxWithdrawable": max_withdrawable_msat,

        }),
        Err(_) => build_lnurl_response_err("SystemTime before UNIX EPOCH"),
    }
}

#[allow(unused_variables)]
#[get("/<network>/withdraw?<k1>&<pr>")]
pub(crate) async fn withdraw_pay_invoice(
    ls_sdk_mainnet: &State<MainnetWallet>,
    ls_sdk_testnet: &State<TestnetWallet>,
    network: ChosenNetwork,
    pr: String,
    k1: String,
) -> Result<Value, Value> {
    let ls_sdk = match network {
        ChosenNetwork::Mainnet => &ls_sdk_mainnet.0,
        ChosenNetwork::Testnet => &ls_sdk_testnet.0,
    };

    // TODO Check k1
    ls_sdk
        .send_payment(&pr)
        .map(|res| json!({"txid": res.txid }))
        .map_err(|err| build_lnurl_response_err(&format!("Failed to pay invoice: {err}")))
}
