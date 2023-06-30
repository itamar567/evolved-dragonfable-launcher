#![feature(lazy_cell)]
#![windows_subsystem = "windows"]

use std::{fs, process, thread};
use std::process::Command;
use std::sync::LazyLock;
use std::time::Duration;
use crate::config::{LOCAL_SERVER_ADDR, PROJECT_DIRS};
use axum::extract::Path;
use axum::routing::{get, post};
use axum::Router;
use http::{HeaderMap, Uri};
use tower_http::trace::TraceLayer;
use tracing_core::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use reqwest::Client;

mod config;
mod encryption;
mod files;
mod server;
mod character;

static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::main]
pub async fn main() {
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

    let pid_file = &PROJECT_DIRS.cache_dir().join("pid");

    if let Ok(Ok(old_process_id)) = fs::read_to_string(pid_file).map(|pid| pid.parse::<u32>()) {
        #[cfg(target_os = "linux")] {
            Command::new("kill")
                .arg(old_process_id.to_string())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        #[cfg(target_os = "windows")] {
            Command::new("taskkill")
                .arg("/PID")
                .arg(old_process_id.to_string())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        thread::sleep(Duration::from_millis(50));
    }
    fs::write(pid_file, process::id().to_string()).unwrap();

    #[cfg(not(debug_assertions))] {
        tokio::spawn(axum::Server::bind(&LOCAL_SERVER_ADDR.parse().unwrap()).serve(app.into_make_service()));
        Command::new("./ui").spawn().unwrap().wait().unwrap();
    }
    #[cfg(debug_assertions)] {
        axum::Server::bind(&LOCAL_SERVER_ADDR.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
    }
}
