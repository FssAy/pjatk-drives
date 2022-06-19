use super::*;
use crate::web::{HtmlBody, Page};
use crate::CONFIG;

/// This endpoint loads an html page with the file manager
pub struct MainEndpoint;

#[async_trait]
impl Endpoint for MainEndpoint {
    fn method(&self) -> Option<Method> {
        Some(Method::GET)
    }

    fn path(&self) -> &'static str {
        "main"
    }

    fn classification(&self) -> EndpointClassification {
        EndpointClassification::HTML
    }

    async fn call(&self, _meta: Parts, _body: Body, _address: SocketAddr) -> Response<Body> {
        let html_main = HtmlBody::new(Page::Main)
            .var("host", &CONFIG.host)
            .var("api", CONFIG.api_version.to_string());

        return Response::new(html_main.into());
    }
}
