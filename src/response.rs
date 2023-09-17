use std::fs::File;
use std::io::{BufReader, Read, Result, Write};
use std::{io::BufWriter, net::TcpStream};

pub struct Response {
    writer: BufWriter<TcpStream>,
}

pub fn status(code: i32) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

impl Response {
    pub fn new(client: TcpStream) -> Self {
        Self {
            writer: BufWriter::new(client),
        }
    }

    pub fn into_inner(self) -> Result<TcpStream> {
        match self.writer.into_inner() {
            Ok(client) => Ok(client),
            Err(e) => Err(e.into_error()),
        }
    }

    pub fn write_status(&mut self, code: i32) -> Result<usize> {
        self.writer
            .write(format!("HTTP/1.1 {} {}\r\n", code, status(code)).as_bytes())
    }
    pub fn write_header(&mut self, key: &str, val: &str) -> Result<usize> {
        self.writer
            .write(format!("{}: {}\r\n", key, val).as_bytes())
    }
    pub fn write_body(&mut self, val: &[u8]) -> Result<usize> {
        self.write_header("content-length", &val.len().to_string())?;
        self.writer.write(b"\r\n")?;
        self.writer.write(val)
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

    pub fn write_file(&mut self, path: &str) -> Result<usize> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        let mut reader = BufReader::new(&mut file);
        reader.read_to_end(&mut buf)?;

        self.write_header("content-type", &format!("{}", self.mime_type(path)))?;
        self.write_body(&buf)
    }

    pub fn sendfile(&mut self, code: i32, path: &str) -> Result<()> {
        self.write_status(code)?;
        self.write_file(path)?;
        self.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}
