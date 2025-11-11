use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use tokio::sync::mpsc;

mod engine;
mod handlers;
mod messages;
mod orderbook;
mod state;
mod types;
mod utils;

use engine::run_orderbook_engine;
use handlers::auth::UserStore;
use state::AppState;
use utils::jwt_validator;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("🚀 Starting Orderbook System...");

    // Create mpsc channel for orderbook commands
    let (orderbook_tx, orderbook_rx) = mpsc::channel(100);

    // Start orderbook engine in background
    tokio::spawn(run_orderbook_engine(orderbook_rx));

    // Create shared state
    let app_state = web::Data::new(AppState::new(orderbook_tx));
    let user_store = web::Data::new(UserStore::new());

    // Create JWT auth middleware
    let auth = HttpAuthentication::bearer(jwt_validator);

    println!("📊 Orderbook engine started");
    println!("🌐 Starting HTTP server on http://127.0.0.1:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .app_data(user_store.clone())
            // Public routes
            .service(
                web::scope("/api")
                    // Health check
                    .service(handlers::health)
                    // Auth routes (no auth required)
                    .service(
                        web::scope("/auth")
                            .service(handlers::signup)
                            .service(handlers::signin)
                    )
                    // Market data (no auth required)
                    .service(handlers::get_orderbook)
                    // Protected routes (auth required)
                    .service(
                        web::scope("/orders")
                            .wrap(auth.clone())
                            .service(handlers::create_limit_order)
                            .service(handlers::create_market_order)
                            .service(handlers::cancel_order)
                    )
                    .service(
                        web::scope("/user")
                            .wrap(auth.clone())
                            .service(handlers::get_balance)
                            .service(handlers::onramp)
                    )
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
