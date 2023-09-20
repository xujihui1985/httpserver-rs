
use std::io::{BufReader, Read};

use crate::{async_net::client::TcpClient, async_io::reactor};

pub struct Response {
    client: TcpClient
}

pub fn status_code(code: i32) -> i32 {
    match code {
        200 | 400 | 404 => code,
        _ => 501
    }
}

pub fn status(code: i32) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Not Implemented"
    }
}

impl Response {
    pub fn new(client: TcpClient) -> Self {
        Response { client }
    }

    pub fn read_response_file(&self, path: &str) -> Vec<u8> {
        let file = std::fs::File::open(String::from(path)).unwrap();
        let mut buf = Vec::new();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut buf).unwrap();
        buf
    }

    pub fn mime_type(&self, key: &str) -> &str {
        if let Some((_, ext)) = key.rsplit_once(".") {
            match ext {
                "html" => "text/html; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "js" => "text/javascript; charset=utf-8",
                "png" => "image/png",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "svg" => "image/svg+xml",
                "ico" => "image/x-icon",
                "json" => "application/json",
                "pdf" => "application/pdf",
                "zip" => "application/zip",
                "tar" => "application/x-tar",
                "gz" => "application/gzip",
                "mp3" => "audio/mpeg",
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                "ogg" => "audio/ogg",
                "ogv" => "video/ogg",
                "wav" => "audio/wav",
                "webp" => "image/webp",
                "txt" => "text/plain; charset=utf-8",
                _ => "application/octet-stream",
            }
        } else {
            "application/octet-stream"
        }
    }

    pub async fn send_file(&mut self, code: i32, path: &str) -> std::io::Result<()> {
            let contents = self.read_response_file(path.clone());
            let len = contents.len();
            let mime_type = self.mime_type(path);
            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                status_code(code),
                status(code),
                mime_type,
                len
            );
            let bytes = response.as_bytes();
            self.client.write(&bytes).await?;
            self.client.write(&contents).await?;
            self.client.flush();
            Ok(())
    }
}