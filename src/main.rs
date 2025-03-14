mod config;
mod model;
mod auth;
mod routes;

use actix_cors::Cors;
use actix_web::{get, http::header, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use config::config::{config_scope, Config};
use dotenv::dotenv;
use sqlx::{pool, postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
    env: Config,
}

#[get("/api/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "JWT Authentication in Rust using Actix-web working okay.";
    HttpResponse::Ok().json(serde_json::json!({"status": "success", "message": MESSAGE}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Connection to database successful.");
            pool
        }
        Err(e) => {
            println!("Failed to connect to database: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Server started successfully...");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT
            ]);
            // .supports_credentials();

        App::new()
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                env: config.clone(),
            }))
            .configure(config_scope)
            .wrap(cors)
            .wrap(Logger::default())
            .service(health_checker_handler)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}