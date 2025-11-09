use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};

#[get("/orderbook")]
async fn orderbook() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
