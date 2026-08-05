//Server
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use std::convert::Infallible;
use std::fmt::Display;
use std::fmt::Formatter;
use std::net::SocketAddr;

//Client
use futures::executor::block_on;
use hyper::body::HttpBody as _;
use hyper::client::HttpConnector;
use hyper::Client;
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::io;
use tokio::io::{stdout, AsyncWriteExt as _};
use tonic::transport::Uri;

//Alias type
type ResultSolo<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Client
//-------

/**
Http client running in [Http2]
* We parser an Http Uri with the [ip:port/endpoint] that return a [Result] with the [Uri].
    We use [?] to extract the value from Result, since we result a Result.
* We use [Client::builder] to create [Client] specifying idle timeout, set to work only in [http2]
    [retry_canceled_requests] to retry a request in case the idle timeout is reach.
* We use [client.get] passing the [uri] to make the request, and return a [Future] of [Result] of [Response<Body>].
    We use [await?] to await for the response, and extract the Body value from Result.
* A [Response] it contains the [headers] and [status] of the request.
* Using <await> we are able to await until the response is available, and then using [unwrap]
    we are able to extract from the [option] the [Result] with the Bytes, so finally using [?]
    we extract the bytes to be used in the String response.
*/
pub async fn run_client() -> ResultSolo<()> {
    let uri = "http://localhost:1981/hello".parse()?;

    let client: Client<HttpConnector, Body> = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .http2_only(true)
        .retry_canceled_requests(true)
        .build_http();

    let mut res = client.get(uri).await?;
    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    let bytes = res.data().await.unwrap()?;
    let result = String::from_utf8(bytes.into_iter().collect()).expect("");
    println!("\n\nnResponse:{}", result);
    Ok(())
}

// Server
//-------

/**
Function to create a Http Server and Service.

* We use [SocketAddr::from] to pass a tuple of [ip] array and [port]
* Using [make_service_fn] we implement a function that receive an [AddStream] and return function that return a
    [Future] of [Result<Response<Body>, Infallible>]
* Once we have the service function, use it to be [bind] with the [SocketAddress] using [serve] function.
* We can force only [http2] protocol is allowed with [http2_only] as true.
* Inside the async function we pass to [service_fn] the implementation of our service [create_service] which
    receive a [Request<Body>], and return [Result<Response<Body>, Infallible>].
* Then with the response [server] we await forever.
 */
pub async fn run_server() {
    println!("Preparing Service...");
    let port = 1981;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::bind(&addr)
        .http2_only(true)
        .serve(make_service_fn(|_conn| async {
            println!("New request received.");
            Ok::<_, Infallible>(service_fn(create_service))
        }));
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

/**
Function to declare service routing and response.
* We use pattern matching to match the [method] of the request, and the [uri]
* Once we're in the specific handle, we can set body response using [body_mut] over pointer [response]
 */
async fn create_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => {
            *response.body_mut() = Body::from("In the near future, we will implement /world");
        }
        (&Method::POST, "/world") => *response.status_mut() = StatusCode::NOT_IMPLEMENTED,
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

// DSL use case Rest Connector
// ----------------------------
trait RestClient {}

struct RestConnector {
    uri: Uri,
    client: Client<HttpConnector, Body>,
}

#[derive(Debug)]
struct RestError(String);

impl Display for RestError {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Err(std::fmt::Error)
    }
}

impl Error for RestError {}

impl From<hyper::Error> for RestError {
    fn from(_: hyper::Error) -> Self {
        self::RestError("hyper::Error".to_string())
    }
}

impl RestConnector {

    fn connect(uri_path: String) -> RestConnector {
        let uri = uri_path.parse::<Uri>().unwrap();
        let client: Client<HttpConnector, Body> = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .http2_only(true)
            .retry_canceled_requests(true)
            .build_http();
        RestConnector {
            client: client,
            uri: uri,
        }
    }

    async fn get(&mut self) -> Result<String, RestError> {
        let mut res = self.client.get(self.uri.clone()).await?;
        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        let bytes = res.data().await.unwrap()?;
        let result = String::from_utf8(bytes.into_iter().collect()).expect("");
        println!("\n\nnResponse:{}", result);
        Ok(result)
    }
}

mod test {
    use crate::tokio_http_hyper::RestConnector;

    #[tokio::test]
    async fn server_client_test() {
        let server = tokio::spawn(async {
            super::run_server().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        super::run_client().await.unwrap();

        server.abort();
    }

    #[tokio::test]
    async fn server_client_dsl() {
        let server = tokio::spawn(async {
            super::run_server().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let uri = String::from("http://localhost:1981/hello");
        let mut connector = RestConnector::connect(uri);

        let result = connector.get().await.unwrap();
        println!("Client response {}", result);
    }
}
