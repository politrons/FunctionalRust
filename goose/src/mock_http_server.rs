//Server
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};

//Client
use std::error::Error;
use std::num::NonZeroU16;
use std::thread;
use std::time::Duration;
use hyper::Client;
use hyper::body::HttpBody as _;
use hyper::client::HttpConnector;
use tokio::io;
use tokio::io::{stdout, AsyncWriteExt as _};


pub async fn run_server() {
    println!("Preparing Service...");
    let port = 1981;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::bind(&addr)
        .serve(make_service_fn(|_conn| async {
            println!("New request received.");
            Ok::<_, Infallible>(service_fn(create_service))
        }));
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

async fn create_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => {
            *response.status_mut() = StatusCode::OK;
            *response.body_mut() = Body::from("In the near future, we will implement /world");
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}