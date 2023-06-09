use std::io::{Read, Write};
use axum::body::Bytes;
use byteorder::ByteOrder;
use flate2::read::ZlibDecoder;
use http::{HeaderMap, HeaderValue};
use crate::config::{FLASH_PLAYER_LAUNCH_URL, LOCAL_SERVER_ADDR};

pub fn spoof_headers(headers: HeaderMap, remote_server_url: &str, remote_server_addr: &str) -> HeaderMap {
    let mut spoofed_headers = HeaderMap::new();

    for (name, mut value) in headers {
        if let Some(name) = &name {
            // Replace the `referer` header
            if name.to_string().to_lowercase() == "referer" {
                if let Ok(value_str) = value.to_str() {
                    if let Ok(new_value) =  HeaderValue::from_str(value_str.replace(FLASH_PLAYER_LAUNCH_URL, remote_server_url).as_str()) {
                        value = new_value;
                    }
                }
            }

            // Replace the `host` header
            if name.to_string().to_lowercase() == "host" {
                if let Ok(value_str) = value.to_str() {
                    if let Ok(new_value) =  HeaderValue::from_str(value_str.replace(LOCAL_SERVER_ADDR, remote_server_addr).as_str()) {
                        value = new_value;
                    }
                }
            }

            spoofed_headers.insert(name, value);
        }
    }

    spoofed_headers
}

fn replace_slice<T>(source: &[T], from: &[T], to: &[T]) -> Vec<T>
    where
        T: Clone + PartialEq
{
    let mut result = source.to_vec();
    let from_len = from.len();
    let to_len = to.len();

    let mut i = 0;
    while i + from_len <= result.len() {
        if result[i..].starts_with(from) {
            result.splice(i..i + from_len, to.iter().cloned());
            i += to_len;
        } else {
            i += 1;
        }
    }

    result
}

pub fn replace_da_check_url_in_swf<R: Read>(mut swf: R) -> Result<Bytes, ()> {
    // Compression
    // Only ZLIB compression is supported
    let mut compression_type = [0u8; 3];
    if swf.read_exact(&mut compression_type).is_err() || compression_type != "CWS".as_bytes() {
        return Err(());
    }

    // Version
    let mut version = [0u8];
    if swf.read_exact(&mut version).is_err() {
        return Err(());
    }

    // Length
    let mut length = [0u8; 4]; // u32
    if swf.read_exact(&mut length).is_err() {
        return Err(());
    }

    // Body
    let mut reader = ZlibDecoder::new(swf);
    let mut uncompressed_swf = Vec::new();
    if reader.read_to_end(&mut uncompressed_swf).is_err() {
        return Err(());
    }

    // Replace the string
    // The replaced string must be the same length as the original one
    let mut replaced_swf_body = replace_slice(&uncompressed_swf, "https://dragonfable.battleon.com/game/cf-dacheck.asp".as_bytes(), "http://127.0.0.1:39260/game/cf-dacheck.asp_____https".as_bytes());
    replaced_swf_body = replace_slice(&replaced_swf_body, "http://dragonfable.battleon.com/game/cf-dacheck.asp".as_bytes(), "http://127.0.0.1:39260/game/cf-dacheck.asp_____http".as_bytes());

    dbg!(byteorder::LittleEndian::read_u32(&length));

    // Compression
    // We disable compression due to high transfer speeds
    let mut result = Vec::new();
    if result.write("FWS".as_bytes()).is_err() {
        return Err(());
    }

    // Version
    if result.write(&version).is_err() {
        return Err(());
    }

    dbg!(replaced_swf_body.len() + 8);

    // Length
    let mut new_length = [0u8; 4];
    byteorder::LittleEndian::write_u32(&mut new_length, replaced_swf_body.len() as u32 + 8); // +8 for the header length
    if result.write(&new_length).is_err() {
        return Err(());
    }

    // Body
    if result.write(&replaced_swf_body).is_err() {
        return Err(());
    }

    Ok(Bytes::from(result))
}