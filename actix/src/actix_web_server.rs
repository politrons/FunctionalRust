use std::path::PathBuf;
use actix_files::NamedFile;
use actix_web::{web, App, HttpServer};
use actix_web::{HttpRequest, Result};


/**
* We use [parse] to search for the resource and we return a [PathBuf]
* we use [NamedFile::open] feature to open unwrap the [Result], and return a [Result<NamedFile>]
    which actix framework will use to render that static file.
 */
async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/app")
                .route("/", web::get().to(index)),
        )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}