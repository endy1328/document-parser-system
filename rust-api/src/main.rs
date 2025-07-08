use actix_web::{get, App, HttpServer, Responder};

#[get("/healthz")]
async fn healthz() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(healthz)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
