pub mod endpoints;
mod parsers;
pub mod responses;

use crate::logging::*;
use endpoints::ENDPOINTS;

use hyper::header::{
    ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN,
};
use hyper::{Body, Request, Response, StatusCode};

use std::convert::Infallible;
use std::net::SocketAddr;

/// Handles every request received and calls a specific endpoint
pub async fn handler(
    req: Request<Body>,
    address: SocketAddr,
) -> Result<Response<Body>, Infallible> {
    let (meta, body) = req.into_parts();
    let path = meta.uri.path().to_string();

    debug!("Received {} request for [{}]", meta.method, meta.uri.path());

    if let Some(mut response) = ENDPOINTS.call(meta, body, address).await {
        #[cfg(debug_assertions)]
        {
            let headers = response.headers_mut();
            headers.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, "true".parse().unwrap());
            headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            headers.insert(ACCESS_CONTROL_ALLOW_METHODS, "*".parse().unwrap());
            headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
        }
        Ok(response)
    } else {
        if path.starts_with("/api/") {
            use crate::handler::responses::ErrorMessage;
            return Ok(
                ErrorMessage::new("endpoint not found", StatusCode::NOT_FOUND).to_response(),
            );
        }
        Ok(responses::e404())
    }
}
