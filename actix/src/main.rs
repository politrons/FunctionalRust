mod actix_server;
mod actix_web_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   // actix_server::run_server().await
    actix_web_server::run_server().await

}
