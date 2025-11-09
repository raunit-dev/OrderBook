use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};

#[get("/balance")]
async fn balance() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/onramp")]
async fn onramp(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
