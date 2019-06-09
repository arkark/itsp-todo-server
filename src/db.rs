use std::ops::Deref;

use actix::prelude::{Actor, Handler, Message, SyncContext};
use actix_web::{error, Error};
use chrono::{DateTime, FixedOffset};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};

use crate::model::{NewTask, Task};

type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub struct DbExecutor {
    pub pool: PgPool,
}

impl DbExecutor {
    pub fn get_connection(&self) -> Result<PgPooledConnection, Error> {
        self.pool
            .get()
            .map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct AllTasks;

impl Message for AllTasks {
    type Result = Result<Vec<Task>, Error>;
}

impl Handler<AllTasks> for DbExecutor {
    type Result = Result<Vec<Task>, Error>;

    fn handle(&mut self, _: AllTasks, _: &mut Self::Context) -> Self::Result {
        Task::all(self.get_connection()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Failed to get all tasks"))
    }
}

pub struct InsertTask {
    pub deadline: DateTime<FixedOffset>,
    pub title: String,
    pub memo: String,
}

impl Message for InsertTask {
    type Result = Result<i64, Error>;
}

impl Handler<InsertTask> for DbExecutor {
    type Result = Result<i64, Error>;

    fn handle(&mut self, insert_task: InsertTask, _: &mut Self::Context) -> Self::Result {
        let new_task = NewTask {
            deadline: insert_task.deadline.naive_utc(),
            title: insert_task.title,
            memo: insert_task.memo,
        };
        Task::insert(&new_task, self.get_connection()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Failed to insert a task"))
    }
}

#[derive(Deserialize)]
pub struct SearchTask {
    pub id: i64,
}

impl Message for SearchTask {
    type Result = Result<Task, Error>;
}

impl Handler<SearchTask> for DbExecutor {
    type Result = Result<Task, Error>;

    fn handle(&mut self, search_task: SearchTask, _: &mut Self::Context) -> Self::Result {
        Task::search(search_task.id, self.get_connection()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Failed to search a task"))
    }
}
