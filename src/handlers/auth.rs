use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};

#[post("/signup")]
async fn signup() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/signin")]
async fn signin(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
