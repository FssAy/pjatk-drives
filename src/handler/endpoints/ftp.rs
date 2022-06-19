mod listing;

use super::*;
use crate::cache::{Cache, FtpClientID};

use crate::handler::responses;
use crate::handler::responses::ErrorMessage;
use crate::utils;
use cookie::Cookie;
use hyper::header::COOKIE;
use hyper::http::response::Builder;
use hyper::StatusCode;
use listing::*;
use ssh2::{FileStat, FileType};

use std::path::Path;
use crate::utils::BoolOptional;

/// API endpoint for communicating with the sftp client
/// * List directory entities
/// * Download files
/// * Uploading files (wip)
/// * Creating new directories (wip)
pub struct FTPEndpoint;

#[async_trait]
impl Endpoint for FTPEndpoint {
    fn uri_path_expanded(&self) -> bool {
        true
    }

    fn path(&self) -> &'static str {
        "ftp"
    }

    fn classification(&self) -> EndpointClassification {
        EndpointClassification::API(1)
    }

    async fn call(&self, meta: Parts, _body: Body, _address: SocketAddr) -> Response<Body> {
        // get the ftp client id
        let mut id = String::new();
        if let Some(ftp) = meta.headers.get("ftp") {
            id = ftp.to_str().unwrap_or("").to_string();
        } else if let Some(cookie) = meta.headers.get(COOKIE) {
            if let Ok(cookie) = Cookie::parse(cookie.to_str().unwrap_or("")) {
                id = cookie.value().to_string();
            }
        }

        if id.is_empty() {
            return ErrorMessage::new(
                "no ftp client identification provided",
                StatusCode::UNAUTHORIZED,
            )
            .to_response();
        }
        let id = FtpClientID::new(id);

        // resolve the ftp path
        let mut ftp_path = urlencoding::decode(meta.uri.path().to_string().as_str())
            .unwrap_or_default()
            .to_string();
        if let Some(i) = ftp_path.find("/ftp") {
            ftp_path.replace_range(..i + 4, "");
        } else {
            return ErrorMessage::new("invalid uri path", StatusCode::BAD_REQUEST).to_response();
        }
        let ftp_path = Path::new(&ftp_path);

        // helpful when the dir name contains a dot
        let is_dir = {
            let is_dir: BoolOptional = meta
                .headers
                .get("is-dir")
                .map(|is_file| is_file.to_str().unwrap_or_default() == "true")
                .into();

            use BoolOptional::*;
            match (ftp_path.extension().is_none(), is_dir) {
                (true, Undefined) | (true, False) | (false, False) => false,
                _ => true,
            }
        };

        let as_html: utils::BoolOptional = meta
            .headers
            .get("as-html")
            .map(|as_html| as_html.to_str().unwrap_or_default() == "true")
            .into();

        match &meta.method {
            &Method::GET => {
                // list all files in the directory
                if is_dir {
                    match Cache::ftp_read_dir(id, ftp_path.to_string_lossy()).await {
                        Ok(vec) => {
                            let mut listings: Vec<Listing> =
                                vec.into_iter().map(Listing::from).collect();
                            listings.sort();

                            let data = if as_html.is_true() {
                                ListingHTML::from(listings).data
                            } else {
                                serde_json::to_string(&listings).unwrap()
                            };

                            return Builder::new()
                                .status(StatusCode::OK)
                                .body(Body::from(data))
                                .unwrap();
                        }
                        Err(e) => ErrorMessage::new(
                            "cannot list the directory",
                            StatusCode::SERVICE_UNAVAILABLE,
                        )
                        .error_force(e)
                        .to_response(),
                    }
                }
                // download the file
                else {
                    match Cache::ftp_read_file(id, ftp_path.to_string_lossy()).await {
                        Ok(content_pack) => {
                            return responses::file_content_pack(content_pack, ftp_path);
                        }
                        Err(error) => {
                            return ErrorMessage::new(
                                "cannot download the file",
                                StatusCode::SERVICE_UNAVAILABLE,
                            )
                            .error_force(error)
                            .to_response();
                        }
                    }
                }
            }

            &Method::POST => {
                // upload file
                return ErrorMessage::new(
                    "uploading files not implemented",
                    StatusCode::NOT_IMPLEMENTED,
                )
                .to_response();
            }

            &Method::PATCH => {
                // create dir
                return ErrorMessage::new(
                    "creating dir not implemented",
                    StatusCode::NOT_IMPLEMENTED,
                )
                .to_response();
            }

            _ => {
                return responses::e404();
            }
        }
    }
}
