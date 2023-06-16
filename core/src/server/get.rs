use std::fs;
use std::path::PathBuf;
use crate::config::{PROJECT_DIRS, REMOTE_SERVER_ADDR, REMOTE_SERVER_URL};
use crate::server::stream::{StreamBodySender, get_stream_data_blocking};
use crate::server::utils;
use crate::REQWEST_CLIENT;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, StatusCode, Uri};
use std::sync::mpsc;
use axum::extract::Path;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use crate::server::utils::spoof_headers;
use crate::files;

async fn send_get_request(path: &Uri, headers: HeaderMap) -> reqwest::Response {
    let request_headers = spoof_headers(headers, REMOTE_SERVER_URL, REMOTE_SERVER_ADDR);

    REQWEST_CLIENT
        .get(format!("{REMOTE_SERVER_URL}{path}"))
        .headers(request_headers)
        .send()
        .await
        .unwrap()
}

pub async fn unhandled_get_request(path: Uri, headers: HeaderMap) -> impl IntoResponse {
    let remote_server_response = send_get_request(&path, headers).await;

    Response::builder()
        .status(remote_server_response.status())
        .body(StreamBodySender::new(remote_server_response.bytes_stream(), None, None))
        .unwrap()
}

fn get_cache_file_path(path: &Uri) -> PathBuf {
    let mut relative_path = path.to_string();
    relative_path.remove(0); // Convert the path into a relative path
    PROJECT_DIRS.cache_dir().join(relative_path)
}

async fn get_cached_request(cache_file_path: &PathBuf) -> Option<Response> {
    if cache_file_path.exists() {
        let file = File::open(&cache_file_path).await.unwrap();
        let stream = ReaderStream::new(file);
        let body = StreamBodySender::new(stream, None, None);

        return Some(Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .unwrap()
            .into_response());
    }

    None
}

pub async fn get_request_with_cache(path: Uri, headers: HeaderMap) -> impl IntoResponse {
    let cache_file_path = get_cache_file_path(&path);
    if let Some(response) = get_cached_request(&cache_file_path).await {
        return response
    }

    let remote_server_response = send_get_request(&path, headers).await;

    let (tx, rx) = mpsc::channel();

    let status = remote_server_response.status();
    let stream = StreamBodySender::new(remote_server_response.bytes_stream(), Some(tx), None);

    if status == StatusCode::OK {
        tokio::spawn(async move {
            files::write_file(&cache_file_path, get_stream_data_blocking(rx)).unwrap();
        });
    }

    Response::builder()
        .status(status)
        .body(stream)
        .unwrap()
        .into_response()
}

pub async fn get_game_swf(path: Uri, headers: HeaderMap, _version: Path<String>) -> impl IntoResponse {
    let cache_file_path = get_cache_file_path(&path);
    if cache_file_path.exists() {
        if let Ok(bytes) = fs::read(&cache_file_path) {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(bytes))
                .unwrap()
                .into_response();
        }
    }

    let remote_server_response = send_get_request(&path, headers).await;
    let status = remote_server_response.status();
    let bytes = remote_server_response.bytes().await.unwrap();

    if status == StatusCode::OK {
        let modified_bytes;
        if let Ok(replaced_bytes) = utils::replace_da_check_url_in_swf(bytes.as_ref()) {
            modified_bytes = replaced_bytes;
        } else {
            modified_bytes = bytes;
        }

        let _ = files::write_file(&cache_file_path, &modified_bytes);

        return Response::builder()
            .status(status)
            .body(Body::from(modified_bytes))
            .unwrap()
            .into_response();
    }

    Response::builder()
        .status(status)
        .body(Body::from(bytes))
        .unwrap()
        .into_response()
}