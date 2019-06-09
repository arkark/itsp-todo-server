use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::schema::{tasks, tasks::dsl};

#[derive(Debug, Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub deadline: NaiveDateTime,
    pub title: String,
    pub memo: String,
}

#[derive(Debug, Queryable)]
pub struct Task {
    pub id: i64,
    pub deadline: NaiveDateTime,
    pub title: String,
    pub memo: String,
}

impl Serialize for Task {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let deadline: DateTime<Local> = Local.from_utc_datetime(&self.deadline);
        let mut s = serializer.serialize_struct("Task", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("deadline", &deadline)?;
        s.serialize_field("title", &self.title)?;
        s.serialize_field("memo", &self.memo)?;
        s.end()
    }
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
        dsl::tasks.find(id).get_result(connection)
    }
}
