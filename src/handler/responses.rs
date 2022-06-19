use super::*;
use crate::cache::ftp::FileContentPack;
use crate::handler::parsers::extension_to_mime;
use hyper::http::response::Builder;
use hyper::StatusCode;
use std::error::Error;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    message: String,
    code: u16,
    error: Option<String>,
}

impl ErrorMessage {
    pub fn new(message: impl ToString, code: StatusCode) -> Self {
        Self {
            message: message.to_string(),
            code: code.as_u16(),
            error: None,
        }
    }

    /// Inserts an error message into the structure (only if debug_assertions is enabled)
    pub fn error(mut self, error: impl Error) -> Self {
        #[cfg(debug_assertions)]
        {
            self.error = Some(error.to_string());
        }
        self
    }

    /// Inserts an error message into the structure forcefully
    pub fn error_force(mut self, error: impl ToString) -> Self {
        self.error = Some(error.to_string());
        self
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_response(self) -> Response<Body> {
        Builder::new()
            .status(self.code)
            .body(Body::from(self.to_json()))
            .unwrap()
    }
}

/// Returns response for 404 error
pub fn e404() -> Response<Body> {
    Builder::new()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404"))
        .unwrap()
}

/// Parses FileContentPack into a response
pub fn file_content_pack(content_pack: FileContentPack, ftp_path: &Path) -> Response<Body> {
    let file_name = ftp_path
        .file_name()
        .map(|str| str.to_string_lossy().to_string())
        .unwrap_or(String::from("unknwon_file.bin"));

    let mime = extension_to_mime(
        ftp_path
            .extension()
            .map(|str| str.to_str().unwrap_or_default())
            .unwrap_or_default(),
    );

    Builder::new()
        .status(StatusCode::PARTIAL_CONTENT)
        // .header(CONTENT_TYPE, mime)
        // .header(CONTENT_DISPOSITION, format!("attachment; filename=\"{file_name}\""))
        .header("pack-name", file_name)
        .header("pack-number", content_pack.no)
        .header("pack-futures", content_pack.packs_left)
        .header("pack-size", content_pack.size)
        .header("pack-is-last", content_pack.is_last.to_string())
        .header("pack-mime", mime.to_string())
        .body(Body::from(content_pack.bytes))
        .unwrap()
}
