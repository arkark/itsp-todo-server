use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::NaiveDateTime;

use crate::schema::{
    tasks,
    tasks::dsl
};

#[derive(Debug, Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub deadline: NaiveDateTime,
    pub title: String,
    pub memo: String
}

#[derive(Debug, Queryable, Serialize)]
pub struct Task {
    pub id: i64,
    pub deadline: NaiveDateTime,
    pub title: String,
    pub memo: String
}

impl Task {
    pub fn all(connection: &PgConnection) -> QueryResult<Vec<Task>> {
        dsl::tasks.order(dsl::id.asc()).load::<Task>(connection)
    }

    pub fn insert(new_task: &NewTask, connection: &PgConnection) -> QueryResult<i64> {
        diesel::insert_into(dsl::tasks)
            .values(new_task)
            .returning(dsl::id)
            .get_result(connection)
    }

    pub fn search(id: i64, connection: &PgConnection) -> QueryResult<Task> {
        dsl::tasks.find(id)
            .get_result(connection)
    }
}
