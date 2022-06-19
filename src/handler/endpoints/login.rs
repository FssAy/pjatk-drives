use super::*;
use crate::cache::Cache;
use crate::handler::responses::ErrorMessage;
use crate::logging::*;
use crate::utils::PjatkTools;

use cookie::Cookie;
use hyper::body::HttpBody;
use hyper::header::{ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_EXPOSE_HEADERS, SET_COOKIE};
use hyper::http::response::Builder;
use hyper::StatusCode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginData {
    pub user: String,
    pub password: String,
}

/// API endpoint responsible for:
/// * checking credentials
/// * creating new sftp client
pub struct LoginEndpoint;

#[async_trait]
impl Endpoint for LoginEndpoint {
    fn method(&self) -> Option<Method> {
        Some(Method::POST)
    }

    fn path(&self) -> &'static str {
        "login"
    }

    fn classification(&self) -> EndpointClassification {
        EndpointClassification::API(1)
    }

    async fn call(&self, _meta: Parts, mut body: Body, _address: SocketAddr) -> Response<Body> {
        if let Some(data) = body.data().await {
            match data {
                Ok(bytes) => {
                    match serde_json::from_slice::<LoginData>(bytes.as_ref()) {
                        Ok(login_data) => {
                            if !PjatkTools::are_credentials_valid(&login_data).await {
                                return ErrorMessage::new(
                                    "invalid credentials",
                                    StatusCode::BAD_REQUEST,
                                )
                                .to_response();
                            }

                            match Cache::ftp_connect(login_data).await {
                                Ok(id) => {
                                    info!("{id} connected!");
                                    Builder::new()
                                        .status(StatusCode::OK)
                                        .header(ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")
                                        .header(ACCESS_CONTROL_EXPOSE_HEADERS, SET_COOKIE)
                                        .header(
                                            SET_COOKIE,
                                            Cookie::new("ftp_client", id.as_str()).to_string(),
                                        )
                                        .body(Body::from(id.to_string()))
                                        .unwrap()
                                }
                                Err(error) => {
                                    error!("cannot connect because {error}");
                                    ErrorMessage::new(
                                        "ftp conn failed",
                                        StatusCode::SERVICE_UNAVAILABLE,
                                    )
                                    .error_force(error)
                                    .to_response()
                                }
                            }

                            // ErrorMessage::new("not implemented", StatusCode::NOT_IMPLEMENTED)
                            //     .to_response()
                        }
                        Err(error) => {
                            ErrorMessage::new("invalid login data", StatusCode::BAD_REQUEST)
                                .error_force(error)
                                .to_response()
                        }
                    }
                }
                Err(error) => ErrorMessage::new(
                    "cannot read bytes from the body",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .error(error)
                .to_response(),
            }
        } else {
            ErrorMessage::new("no data in the body", StatusCode::BAD_REQUEST).to_response()
        }
    }
}
