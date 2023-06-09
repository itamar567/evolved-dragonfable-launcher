#![feature(lazy_cell)]
#![windows_subsystem = "windows"]

use std::fs;
use std::fs::File;
use crate::config::{LOCAL_SERVER_ADDR, PROJECT_DIRS};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::Router;
use http::{HeaderMap, Uri};
use native_dialog::{MessageDialog, MessageType};
use tower_http::trace::TraceLayer;
use tracing_core::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod encryption;
mod files;
mod flash_player;
mod server;

#[tokio::main]
pub async fn main() {
    let warning_file_path = PROJECT_DIRS.data_dir().join("removewarning");
    if !warning_file_path.exists() {
        if MessageDialog::new()
            .set_type(MessageType::Warning)
            .set_title("Disclaimer")
            .set_text("This is a 3rd party launcher that is not supported nor endorsed by Artix Entertainment.\
                \nUse this launcher at your own risk.\
                \nClick 'Yes' to use this launcher anyways, and click 'No' to close the app.")
            .show_confirm()
            .unwrap() {

            fs::create_dir_all(PROJECT_DIRS.data_dir()).unwrap();
            File::create(&warning_file_path).unwrap();
        }
        else {
            return;
        }
    }

    let mut flash_player_process = flash_player::start_flash_player().expect("Failed to run Flash Player");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            filter::Targets::new()
                .with_target("tower_http::trace::on_response", Level::DEBUG)
                .with_target("tower_http::trace::on_request", Level::DEBUG)
                .with_target("tower_http::trace::make_span", Level::DEBUG)
                .with_default(Level::INFO),
        )
        .init();

    let app = Router::new()
        // GET requests
        .route("/game/gamefiles/game:version.swf", get(server::get::get_game_swf))
        .route("/game/gamefiles/*_", get(|path: Uri, headers: HeaderMap, _: Path<String>| server::get::get_request_with_cache(path, headers)))
        .route("/game/DFLoader.swf", get(server::get::get_request_with_cache))

        // POST requests
        .route("/game/cf-questload.asp", post(server::post::quest_load))
        .route("/game/cf-loadtowninfo.asp", post(server::post::load_town_info))
        .route("/game/cf-interfaceload.asp", post(server::post::interface_load))
        .route("/game/cf-shopload.asp", post(server::post::shop_load))
        .route("/game/cf-mergeshopload.asp", post(server::post::merge_shop_load))
        .route("/game/cf-loadhouseitemshop.asp", post(server::post::load_house_item_shop))
        .route("/game/cf-loadwarvars.asp", post(server::post::load_war_vars))
        .route("/game/cf-dacheck.asp_____https", post(server::post::da_check_https))
        .route("/game/cf-dacheck.asp_____http", post(server::post::da_check_http))

        .fallback(get(server::get::unhandled_get_request).post(server::post::unhandled_post_request))
        .layer(TraceLayer::new_for_http());

    tokio::spawn(axum::Server::bind(&LOCAL_SERVER_ADDR.parse().unwrap()).serve(app.into_make_service()));

    flash_player_process.wait().unwrap();
}
