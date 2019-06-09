extern crate actix;
extern crate actix_web;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

use actix_web::server;
use log::debug;

mod api;
mod app;
mod db;
mod model;
mod schema;

const DB_THREAD_NUM: usize = 3;

fn main() {
    std::env::set_var("RUST_LOG", "itsp_todo_server=debug,actix_web=info");
    env_logger::init();

    dotenv::dotenv().expect("Failed to read .env file");

    let system = actix::System::new("itsp-todo-app");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not found");
    let addr = app::build_addr(&db_url, DB_THREAD_NUM);

    debug!("Starting the server");
    let port = std::env::var("SERVER_PORT").expect("SERVER_PORT is not found");
    server::new(move || app::build_app(&addr))
        .bind(format!("localhost:{}", port))
        .expect(&format!("Cannot bind to port {}", port))
        .start();

    let _ = system.run();
}
