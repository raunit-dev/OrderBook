use actix_web::{App, HttpResponse, HttpServer, Responder, delete, post, web};

#[post("/limit")]
async fn limit() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/market")]
async fn market(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[delete("/cancel")]
async fn cancel(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
