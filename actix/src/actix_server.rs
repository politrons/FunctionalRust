use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web::http::header::HeaderValue;

/**
Those tags inform [actix] framework about the [method] and [endpoint]
Any endpoint is able to receive [HttpRequest] which contains all information about the
request (Method, Uri, headers..)
 */
#[get("/hello")]
async fn hello(req: HttpRequest) -> impl Responder {
    match req.headers().get("hello_header") {
        None => HttpResponse::Ok().body("Hello world!"),
        Some(header_value) => match header_value.to_str().ok() {
            Some(_) => HttpResponse::Ok().body("Hello world! with header"),
            None => HttpResponse::Ok().body("Hello world!")
        }
    }
}

/**
In case of [POST/PUT] request, the endpoint can mark that expect to receive a [req_body] in [String]
format to internally it can deserialize into String.
 */
#[post("/hello_body")]
async fn hello_with_body(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_routing() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

/**
Actix framework allow to build Http Server that by default accept Http 1/2 protocols.
Using builder [HttpServer] we are to run he server passing:
*   [App] which is created using a builder, and for that App, we can:
**  [route] traffic specifying the details of routing, with [path] and method [web::get::post:put::delete]
**  [service] which use [meta-data tags] in the function to specify all the request details.

Once we have the [HttpServer] we can [bind] an ip and port and [run]

 */
pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/hello_routing", web::get().to(manual_routing))
            .service(hello)
            .service(hello_with_body)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}