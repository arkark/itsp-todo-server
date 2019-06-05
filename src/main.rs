extern crate actix;
extern crate actix_web;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

use actix::SyncArbiter;
use actix_web::middleware::Logger;
use actix_web::{http, server, App};
use log::debug;

mod api;
mod db;
mod model;
mod schema;

const NUM_DB_THREADS: usize = 3;

fn main() {
    std::env::set_var("RUST_LOG", "itsp_todo_server=debug,actix_web=info");
    env_logger::init();

    dotenv::dotenv().expect("Failed to read .env file");

    let system = actix::System::new("itsp-todo-app");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let pool = db::init_pool(&database_url).expect("Failed to create a pool");
    let addr = SyncArbiter::start(NUM_DB_THREADS, move || db::DbExecutor {
        pool: pool.clone(),
    });

    let app = move || {
        debug!("Constructing the app");

        let state = api::AppState { db: addr.clone() };

        App::with_state(state)
            .middleware(Logger::default())
            .route("/api/v1/event", http::Method::POST, api::post_task)
            .route("/api/v1/event", http::Method::GET, api::get_all_tasks)
            .route("/api/v1/event/{id}", http::Method::GET, api::get_task)
    };

    debug!("Starting the server");
    let port = 8080;
    server::new(app)
        .bind(format!("localhost:{}", port))
        .expect(&format!("Cannot bind to port {}", port))
        .start();

    let _ = system.run();
}
