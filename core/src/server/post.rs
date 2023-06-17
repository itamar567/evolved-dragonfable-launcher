use std::io::Cursor;
use crate::config::{PROJECT_DIRS, REMOTE_SERVER_ADDR, REMOTE_SERVER_URL};
use crate::{encryption, files};
use crate::server::stream::{StreamBodySender, get_stream_data_blocking};
use crate::REQWEST_CLIENT;
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, StatusCode, Uri};
use roxmltree::Document;
use std::sync::mpsc;
use tokio_util::io::ReaderStream;
use crate::server::utils::spoof_headers;

async fn send_post_request_to_url(url: String, headers: HeaderMap, form: String, remote_server_url: &str, remote_server_addr: &str) -> reqwest::Response {
    let headers = spoof_headers(headers, remote_server_url, remote_server_addr);

    REQWEST_CLIENT
        .post(url)
        .headers(headers)
        .body(form)
        .send()
        .await
        .unwrap()
}

async fn send_post_request(path: Uri, headers: HeaderMap, form: String) -> reqwest::Response {
    send_post_request_to_url(format!("{REMOTE_SERVER_URL}{path}"), headers, form, REMOTE_SERVER_URL, REMOTE_SERVER_ADDR).await
}

pub async fn unhandled_post_request(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    let remote_server_response = send_post_request(path.clone(), headers, form).await;
    Response::builder()
        .status(remote_server_response.status())
        .body(StreamBodySender::new(remote_server_response.bytes_stream(), None, Some(format!("{REMOTE_SERVER_URL}{path}"))))
        .unwrap()
}

async fn post_request_to_url_with_cache<F>(url: String, dir: String, headers: HeaderMap, form: String, remote_server_url: String, remote_server_addr: String, id_extractor: F) -> impl IntoResponse
where
    F: Fn(Document) -> String,
{
    let form_data = if let Ok(decrypted_data) = encryption::decrypt(
        form.replace("<ninja2>", "")
            .replace("</ninja2>", "")
            .as_str(),
    ) {
        decrypted_data
    } else {
        form.clone()
    };

    if let Ok(doc) = Document::parse(form_data.as_str()) {
        let id = id_extractor(doc);
        let cache_file_path = PROJECT_DIRS.cache_dir().join(dir).join(id);
        if let Some(data) = files::read_file(&cache_file_path, true) {
            let stream = ReaderStream::new(Cursor::new(data));
            let body = StreamBodySender::new(stream, None, Some(url.clone()));

            // Send the POST request to the remote server in the background, and update the cache
            tokio::spawn(async move {
                let remote_server_response = send_post_request_to_url(url, headers, form, remote_server_url.as_str(), remote_server_addr.as_str()).await;
                if remote_server_response.status() == StatusCode::OK {
                    files::write_file(&cache_file_path, remote_server_response.bytes().await.unwrap(), true).unwrap();
                }
            });

            return Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .unwrap()
                .into_response();
        }

        let remote_server_response = send_post_request_to_url(url.clone(), headers, form, remote_server_url.as_str(), remote_server_addr.as_str()).await;

        let (tx, rx) = mpsc::channel();

        let status = remote_server_response.status();
        let stream = StreamBodySender::new(remote_server_response.bytes_stream(), Some(tx), Some(url));

        if status == StatusCode::OK {
            tokio::spawn(async move {
                files::write_file(&cache_file_path, get_stream_data_blocking(rx), true).unwrap();
            });
        }

        Response::builder()
            .status(status)
            .body(stream)
            .unwrap()
            .into_response()
    } else {
        println!("Couldn't parse client data as XML");
        let remote_server_response = send_post_request_to_url(url.clone(), headers, form, remote_server_url.as_str(), remote_server_addr.as_str()).await;

        Response::builder()
            .status(remote_server_response.status())
            .body(StreamBodySender::new(
                remote_server_response.bytes_stream(),
                None,
                Some(url),
            ))
            .unwrap()
            .into_response()
    }
}

async fn post_request_with_cache<F>(path: Uri, headers: HeaderMap, form: String, id_extractor: F) -> impl IntoResponse
    where
        F: Fn(Document) -> String,
{
    let mut relative_path = path.to_string();
    relative_path.remove(0);

    post_request_to_url_with_cache(format!("{REMOTE_SERVER_URL}{path}"), relative_path, headers, form, REMOTE_SERVER_URL.to_string(), REMOTE_SERVER_ADDR.to_string(), id_extractor).await
}

pub async fn quest_load(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        let mut result = doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intCharID")
            .unwrap()
            .text()
            .unwrap()
            .to_string();

        let id = doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intQuestID")
            .unwrap()
            .text()
            .unwrap();

        result.push('/');
        result.push_str(id);

        result
    })
    .await
}

pub async fn load_town_info(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        let mut result = doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intCharID")
            .unwrap()
            .text()
            .unwrap()
            .to_string();

        let id = doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intTownID")
            .unwrap()
            .text()
            .unwrap();

        result.push('/');
        result.push_str(id);

        result
    })
    .await
}

pub async fn interface_load(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intInterfaceID")
            .unwrap()
            .text()
            .unwrap()
            .to_string()
    })
    .await
}

pub async fn shop_load(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intShopID")
            .unwrap()
            .text()
            .unwrap()
            .to_string()
    })
    .await
}

pub async fn merge_shop_load(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intMergeShopID")
            .unwrap()
            .text()
            .unwrap()
            .to_string()
    })
    .await
}

pub async fn load_house_item_shop(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |doc| {
        doc.root_element()
            .children()
            .find(|node| node.tag_name().name() == "intHouseItemShopID")
            .unwrap()
            .text()
            .unwrap()
            .to_string()
    })
    .await
}

pub async fn load_war_vars(path: Uri, headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_with_cache(path, headers, form, |_| "vars".to_string()).await
}

pub async fn da_check_https(headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_to_url_with_cache("https://dragonfable.battleon.com/game/cf-dacheck.asp".to_string(),
                                   "game/cf-dacheck.asp_https".to_string(), headers, form, "https://dragonfable.battleon.com".to_string(), "dragonfable.battleon.com".to_string(), |doc| {
            doc.root_element()
                .children()
                .find(|node| node.tag_name().name() == "intCharID")
                .unwrap()
                .text().unwrap()
                .to_string()
    }).await
}

pub async fn da_check_http(headers: HeaderMap, form: String) -> impl IntoResponse {
    post_request_to_url_with_cache("http://dragonfable.battleon.com/game/cf-dacheck.asp".to_string(),
                                   "game/cf-dacheck.asp_http".to_string(), headers, form, "http://dragonfable.battleon.com".to_string(), "dragonfable.battleon.com".to_string(), |doc| {
            doc.root_element()
                .children()
                .find(|node| node.tag_name().name() == "intCharID")
                .unwrap()
                .text().unwrap()
                .to_string()
        }).await
}