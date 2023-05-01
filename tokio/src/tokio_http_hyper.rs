//Server
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};

//Client
use std::error::Error;
use std::thread;
use hyper::Client;
use hyper::body::HttpBody as _;
use tokio::io;
use tokio::io::{stdout, AsyncWriteExt as _};

//Alias type
type ResultSolo<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Client
//-------
pub async fn run_client() -> ResultSolo<()> {

// Parse an `http::Uri`...
    let uri = "http://localhost:1981/hello".parse()?;

    let client = Client::new();

    let mut res = client.get(uri).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    println!("\n\nDone!");

    Ok(())
}


// Server
//-------

/**
Function to create a Http Server.

*/
pub async fn run_server() {
    println!("Preparing Service...");
    let port = 1981;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        println!("New request received.");
        Ok::<_, Infallible>(service_fn(create_service))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

async fn create_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => {
            *response.body_mut() = Body::from("Try POSTing data to /echo");
        },
        (&Method::POST, "/echo") => {
            // we'll be back
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
    };

    Ok(response)
}