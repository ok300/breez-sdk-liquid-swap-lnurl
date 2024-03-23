mod pay;
mod withdraw;

use std::sync::Arc;

use anyhow::Result;
use breez_sdk_liquid::{Network, Wallet};
use chrono::prelude::*;
use env_logger::{Builder, Env};
use figment::providers::{Format, Toml};
use figment::Figment;
use qrcode_rs::{render::unicode, EcLevel, QrCode};
use rocket::http::ContentType;
use rocket::request::FromParam;
use rocket::response::Redirect;
use rocket::response::Responder;
use rocket::serde::json::{serde_json::json, Value};
use rocket::serde::Deserialize;
use rocket::uri;
use rocket::{fairing::AdHoc, get, response, routes, Request, Response, State};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct AppConfig {
    ls_sdk_data_dir: String,
    mnemonic: String,
    domain: String,
    min_sendable_msat: u64,
    max_sendable_msat: u64,
}

struct MainnetWallet(Arc<Wallet>);
struct TestnetWallet(Arc<Wallet>);

#[rocket::main]
#[allow(unused_must_use)]
pub async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let figment = Figment::new().merge(Toml::file("config.toml"));
    let config: AppConfig = figment.extract().expect("Failed to parse config");

    let ls_sdk_mainnet = Wallet::init(
        &config.mnemonic,
        Some(format!("{}-mainnet", &config.ls_sdk_data_dir)),
        Network::Liquid,
    )?;
    let ls_sdk_testnet = Wallet::init(
        &config.mnemonic,
        Some(format!("{}-testnet", &config.ls_sdk_data_dir)),
        Network::LiquidTestnet,
    )?;
    // info!("[sdk] Node info: {:#?}", sdk.node_info()?);

    let config_managed_state = Arc::new(config);
    let sdk_managed_state_mainnet = MainnetWallet(ls_sdk_mainnet);
    let sdk_managed_state_testnet = TestnetWallet(ls_sdk_testnet);

    let _result = rocket::build()
        .mount(
            "/",
            routes![
                index,
                index_with_network,
                pay::get_lnurl_params,
                pay::get_invoice,
                withdraw::get_lnurl_withdraw_params,
                withdraw::withdraw_pay_invoice
            ],
        )
        .attach(AdHoc::on_ignite("Manage State", |rocket| async move {
            rocket
                .manage(config_managed_state)
                .manage(sdk_managed_state_mainnet)
                .manage(sdk_managed_state_testnet)
        }))
        .launch()
        .await;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum ChosenNetwork {
    Mainnet,
    Testnet,
}
impl<'r> FromParam<'r> for ChosenNetwork {
    type Error = ();

    fn from_param(param: &'r str) -> std::result::Result<Self, Self::Error> {
        match param.to_lowercase().as_str() {
            "mainnet" => Ok(ChosenNetwork::Mainnet),
            _ => Ok(ChosenNetwork::Testnet),
        }
    }
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!("/testnet"))
}

#[get("/<network>")]
fn index_with_network(
    network: ChosenNetwork,
    config: &State<Arc<AppConfig>>,
    ls_sdk_mainnet: &State<MainnetWallet>,
    ls_sdk_testnet: &State<TestnetWallet>,
) -> HtmlContent {
    let ls_sdk = match network {
        ChosenNetwork::Mainnet => &ls_sdk_mainnet.0,
        ChosenNetwork::Testnet => &ls_sdk_testnet.0,
    };

    let ln_address = crate::pay::get_ln_address(config, network);
    let lnurl_pay_qr = crate::pay::get_lnurl_pay_qr(config, network);

    let withdraw_link = crate::withdraw::get_lnurl_withdraw_link(config, network);
    let withdraw_qr = crate::withdraw::get_lnurl_withdraw_qr(config, network);

    let balance_sat = ls_sdk.get_info(true).unwrap().balance_sat;
    let payments_str = ls_sdk
        .list_payments(true, true)
        .unwrap()
        .iter()
        .map(|tx| {
            let ts = match tx.timestamp {
                Some(t) => parse_timestamp(t).to_string(),
                None => format!("(None) {}", " ".repeat(12)),
            };
            let payment_type: String = format!("{:?}", tx.payment_type);
            let padded_amount = format!("{:width$} sat", tx.amount_sat, width = 10);
            format!(
                "{ts} | {} | {padded_amount}",
                format_args!("{:width$}", payment_type, width = 15)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let pay_header = format!("{} [{network:?}] LNURL Pay & LN Address {}", r(3), r(41));
    let withdraw_header = format!("{} [{network:?}] LNURL Withdraw {}", r(3), r(49));
    let balance_header = format!("{} [{network:?}] Balance {}", r(3), r(56));

    let pay_section = format!("{pay_header}\n\n{lnurl_pay_qr}\n\n{ln_address}");
    let withdraw_section = format!("{withdraw_header}\n\n{withdraw_qr}\n\n{withdraw_link}");
    let balance_section = format!("{balance_header}\n\n{balance_sat} sat\n\n{payments_str}");

    let link = get_header(network);
    let monospaced_content = format!("{pay_section}\n\n{withdraw_section}\n\n{balance_section}");

    let r = format!("{link}\n\n<pre>{monospaced_content}</pre>");
    HtmlContent(r)
}

fn get_header(network: ChosenNetwork) -> String {
    let links = match network {
        ChosenNetwork::Mainnet => "<a href=\"/testnet\">Testnet</a> | Mainnet",
        ChosenNetwork::Testnet => "Testnet | <a href=\"/mainnet\">Mainnet</a>",
    };
    format!("<pre><h3>{links}</h3></pre>")
}

struct HtmlContent(String);

impl<'r> Responder<'r, 'static> for HtmlContent {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::HTML)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}

fn build_lnurl_response_err(err_details: &str) -> Value {
    json!({ "status": "ERROR", "reason": err_details })
}

fn parse_timestamp(ts: u32) -> String {
    let dt = DateTime::from_timestamp(ts as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

// Repeat pattern used in formatting headlines
fn r(i: usize) -> String {
    "_".repeat(i)
}
