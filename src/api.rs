use actix::prelude::Addr;
use actix_web::{
    HttpRequest, HttpResponse,
    FutureResponse, AsyncResponder,
    Path, Json
};
use futures::Future;

use crate::db::{DbExecutor, AllTasks, InsertTask, SearchTask};
use crate::model::Task;

pub struct AppState {
    pub db: Addr<DbExecutor>
}

pub mod post_task_response {
    use serde::ser::{Serialize, Serializer, SerializeStruct};

    pub struct Success {
        pub id: i64
    }

    impl Serialize for Success {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut s = serializer.serialize_struct("Success", 3)?;
            s.serialize_field("status", "success")?;
            s.serialize_field("message", "registered")?;
            s.serialize_field("id", &self.id)?;
            s.end()
        }
    }

    pub struct Failure;

    impl Serialize for Failure {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut s = serializer.serialize_struct("Failure", 2)?;
            s.serialize_field("status", "success")?;
            s.serialize_field("message", "invalid date format")?;
            s.end()
        }
    }
}

pub fn post_task(
    request: HttpRequest<AppState>,
    task: Json<InsertTask>
) -> FutureResponse<HttpResponse> {
    request.state()
        .db
        .send(task.into_inner())
        .from_err()
        .and_then(move |res| match res {
            Ok(id) => Ok(
                HttpResponse::Ok().json(
                    post_task_response::Success { id }
                )
            ),
            Err(_) => Ok(
                HttpResponse::BadRequest().json(
                    post_task_response::Failure
                )
            )
        })
        .responder()
}

#[derive(Serialize)]
pub struct AllTasksResponse {
    events: Vec<Task>
}

pub fn get_all_tasks(
    request: HttpRequest<AppState>
) -> FutureResponse<HttpResponse> {
    request.state()
        .db
        .send(AllTasks)
        .from_err()
        .and_then(move |res| match res {
            Ok(tasks) => Ok(
                HttpResponse::Ok().json(
                    AllTasksResponse { events: tasks }
                )
            ),
            Err(e) => Err(e) // InternalServerError
        })
        .responder()
}

pub fn get_task(
    (request, params): (HttpRequest<AppState>, Path<SearchTask>)
) -> FutureResponse<HttpResponse> {
    request.state()
        .db
        .send(params.into_inner())
        .from_err()
        .and_then(move |res| match res {
            Ok(task) => Ok(
                HttpResponse::Ok().json(task)
            ),
            Err(_) => Ok(
                HttpResponse::NotFound().finish()
            )
        })
        .responder()
}
