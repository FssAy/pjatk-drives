pub mod ftp;
pub mod index;
pub mod login;
pub mod main;

pub use index::*;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

use crate::handler::endpoints::ftp::FTPEndpoint;
use crate::handler::endpoints::login::LoginEndpoint;
use crate::handler::endpoints::main::MainEndpoint;
use hyper::http::request::Parts;
use hyper::{Body, Method, Response};
use std::path::PathBuf;
use std::sync::Arc;

use tracing::info;

/// HashMap of endpoint's path and it's trait object
pub type EndpointMap = HashMap<String, Arc<Box<dyn Endpoint>>>;

/// EndpointManager should be unchanged by the entire lifetime of the process
lazy_static! {
    pub static ref ENDPOINTS: EndpointManager = {
        let manager = EndpointManager::new()
            .add(IndexEndpoint)
            .add(MainEndpoint)
            .add(LoginEndpoint)
            .add(FTPEndpoint);

        #[cfg(debug_assertions)]
        {
            manager.info()
        }

        manager
    };
}

/// This enum helps in building proper url path and indicates
/// what action a certain endpoint should perform
#[derive(Copy, Clone, Debug)]
pub enum EndpointClassification {
    None,
    HTML,
    API(u8),
    BIN,
}

impl Display for EndpointClassification {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EndpointClassification::None | EndpointClassification::HTML => "/".into(),
                EndpointClassification::API(version) => format!("/api/{version}/"),
                EndpointClassification::BIN => "/bin/".into(),
            }
        )
    }
}

/// Async trait that needs to be implemented by every endpoint object
#[async_trait]
pub trait Endpoint: Sync + Send {
    /// Set to true if uri path can have additional data
    fn uri_path_expanded(&self) -> bool {
        false
    }

    /// returns which method this endpoint can process.
    /// set it to `None` in order to call this endpoint on
    /// every method or ones set in the `call()` method
    fn method(&self) -> Option<Method> {
        None
    }

    /// returns the uri path of the endpoint.
    fn path(&self) -> &'static str;

    /// returns the endpoint classification so the proper url could be detected
    fn classification(&self) -> EndpointClassification;

    /// returns the full url path to the endpoint
    fn full_path(&self) -> String {
        format!("{}{}", self.classification(), self.path())
    }

    /// Calls the endpoint and should always return a response
    async fn call(&self, meta: Parts, mut body: Body, address: SocketAddr) -> Response<Body>;
}

/// Manages all the available endpoints
#[derive(Default)]
pub struct EndpointManager {
    map: EndpointMap,
}

impl EndpointManager {
    fn new() -> Self {
        Self::default()
    }

    /// Adds a specific endpoint, if the endpoint's path matches path of an existing endpoint,
    /// the old one will be overwritten
    fn add(mut self, endpoint: impl Endpoint + 'static) -> Self {
        self.map
            .insert(endpoint.full_path(), Arc::new(Box::new(endpoint)));
        self
    }

    /// Prints info about all of the endpoints in the map
    fn info(&self) {
        let mut buffer = String::new();
        for (full_path, endpoint) in &self.map {
            buffer = format!("{buffer}\n[{full_path}] Method: {:?}", endpoint.method());
        }
        info!("{buffer}")
    }

    /// Finds matching endpoint and calls it
    pub async fn call(
        &self,
        meta: Parts,
        body: Body,
        address: SocketAddr,
    ) -> Option<Response<Body>> {
        let call_endpoint = |endpoint: Arc<Box<dyn Endpoint>>,
                             meta: Parts,
                             body: Body,
                             address: SocketAddr| {
            async move {
                if let Some(method) = endpoint.method() {
                    if method == meta.method {
                        return Some(endpoint.call(meta, body, address).await);
                    }
                } else {
                    return Some(endpoint.call(meta, body, address).await);
                }

                None
            }
        };

        let meta_uri = meta.uri.path();
        for (path, endpoint) in &self.map {
            if endpoint.uri_path_expanded() && meta_uri.starts_with(path) {
                return call_endpoint(endpoint.clone(), meta, body, address).await;
            } else if meta_uri == path {
                return call_endpoint(endpoint.clone(), meta, body, address).await;
            }
        }

        None
    }
}
