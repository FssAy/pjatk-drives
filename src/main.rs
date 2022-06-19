#[macro_use]
extern crate tokio;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate async_trait;

mod cache;
mod config;
mod handler;
mod logging;
pub mod utils;
mod web;

use handler::handler;
pub use logging::*;

lazy_static! {
    pub static ref CONFIG: config::Config = config::Config::new().unwrap();
}

#[tokio::main]
async fn main() {
    init_log();

    cache::Cache::init().await;

    // todo: handle server crash
    if let Err(error) = run_web_server().await {
        error!("CRITICAL ({}).", error);
    } else {
        info!("Closing the web server.");
    }
}

/// Reads config and tries to run the web server,
/// any error is critical and unrecoverable
async fn run_web_server() -> anyhow::Result<()> {
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::Server;
    use std::convert::Infallible;

    let config = &CONFIG;

    let service = make_service_fn(move |conn: &AddrStream| {
        let address = conn.remote_addr();

        let service = service_fn(move |req| async move { handler(req, address).await });

        async move { Ok::<_, Infallible>(service) }
    });

    info!("Running on: {}", &config.host);

    Server::bind(&config.bind).serve(service).await?;
    Ok(())
}
