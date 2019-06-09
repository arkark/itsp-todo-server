use actix::prelude::Addr;
use actix::SyncArbiter;
use actix_web::middleware::Logger;
use actix_web::{http, App};
use log::debug;

use crate::api;
use crate::db;

pub fn build_addr(db_url: &str, db_thread_num: usize) -> Addr<db::DbExecutor> {
    let pool = db::init_pool(&db_url).expect("Failed to create a pool");
    let addr = SyncArbiter::start(db_thread_num, move || db::DbExecutor { pool: pool.clone() });
    addr
}

pub fn build_app(addr: &Addr<db::DbExecutor>) -> App<api::AppState> {
    debug!("Constructing the app");

    let state = api::AppState { db: addr.clone() };

    App::with_state(state)
        .middleware(Logger::default())
        .route("/api/v1/event", http::Method::POST, api::post_task)
        .route("/api/v1/event", http::Method::GET, api::get_all_tasks)
        .route("/api/v1/event/{id}", http::Method::GET, api::get_task)
}
