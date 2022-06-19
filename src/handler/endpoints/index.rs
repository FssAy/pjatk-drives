use super::*;

use crate::web::{HtmlBody, Page};
use crate::CONFIG;

/// HTML endpoint which returns the login page
pub struct IndexEndpoint;

#[async_trait]
impl Endpoint for IndexEndpoint {
    fn method(&self) -> Option<Method> {
        Some(Method::GET)
    }

    fn path(&self) -> &'static str {
        ""
    }

    fn classification(&self) -> EndpointClassification {
        EndpointClassification::HTML
    }

    async fn call(&self, _meta: Parts, _body: Body, _address: SocketAddr) -> Response<Body> {
        let html_login = HtmlBody::new(Page::Login)
            .var("host", &CONFIG.host)
            .var("api", CONFIG.api_version.to_string());

        Response::new(html_login.into())
    }
}
