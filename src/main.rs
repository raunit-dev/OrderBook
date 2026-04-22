use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc;

mod engine;
mod handlers;
mod messages;
mod orderbook;
mod state;
mod types;
mod utils;

use engine::run_orderbook_engine;
use state::AppState;
use utils::jwt_validator;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env (no-op if the file isn't present)
    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("Starting Demo-OrderBook...");

    // --- Postgres: pool + run migrations ---
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");

    println!("Postgres connected, migrations applied");

    // --- Orderbook engine: single owner task + command channel ---
    let (orderbook_tx, orderbook_rx) = mpsc::channel(100);
    tokio::spawn(run_orderbook_engine(orderbook_rx));

    // --- Shared app state: pool + channel sender ---
    let app_state = web::Data::new(AppState::new(orderbook_tx, db_pool));

    // --- JWT auth middleware ---
    let auth = HttpAuthentication::bearer(jwt_validator);

    println!("Demo-OrderBook engine started");
    println!("Starting HTTP server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .service(handlers::health)
                    .service(
                        web::scope("/auth")
                            .service(handlers::signup)
                            .service(handlers::signin),
                    )
                    .service(handlers::get_orderbook)
                    .service(
                        web::scope("/orders")
                            .wrap(auth.clone())
                            .service(handlers::create_limit_order)
                            .service(handlers::create_market_order)
                            .service(handlers::cancel_order),
                    )
                    .service(
                        web::scope("/user")
                            .wrap(auth.clone())
                            .service(handlers::get_balance)
                            .service(handlers::onramp),
                    ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
