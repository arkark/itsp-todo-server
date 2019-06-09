extern crate assert_cmd;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;

use assert_cmd::prelude::*;
use reqwest::{header, StatusCode};

use std::process;
use std::thread;
use std::time;

const SERVER_PORT: &str = "8888";

pub mod todo_response {
    #[derive(Deserialize, Debug)]
    pub struct GetTask {
        pub id: i64,
        pub deadline: String,
        pub title: String,
        pub memo: String,
    }

    impl PartialEq for GetTask {
        fn eq(&self, other: &Self) -> bool {
            use chrono::DateTime;
            let date1 =
                DateTime::parse_from_rfc3339(&self.deadline).expect("Failed to parse deadline");
            let date2 =
                DateTime::parse_from_rfc3339(&other.deadline).expect("Failed to parse deadline");
            self.id == other.id
                && self.title == other.title
                && self.memo == other.memo
                && date1 == date2
        }
    }

    pub mod post_task {
        #[derive(Deserialize, Debug)]
        pub struct Success {
            pub status: String,
            pub message: String,
            pub id: i64,
        }

        #[derive(Deserialize, Debug)]
        pub struct Failure {
            pub status: String,
            pub message: String,
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct GetAllTask {
        pub events: Vec<GetTask>,
    }
}

fn execute_server() -> process::Child {
    process::Command::cargo_bin("itsp-todo-server")
        .unwrap()
        .env(
            "DATABASE_URL",
            "postgresql://user:password@localhost:5432/itsp-todo-app",
        )
        .env("SERVER_PORT", SERVER_PORT)
        .spawn()
        .expect("Failed to execute itsp-todo-server")
}

#[test]
fn server_api() -> Result<(), reqwest::Error> {
    let mut server = execute_server();
    thread::sleep(time::Duration::from_secs(5));

    let base_url = format!("http://localhost:{}", SERVER_PORT);
    let client = reqwest::Client::new();

    let task1 = todo_response::GetTask {
        id: 1,
        deadline: String::from("2019-06-11T14:00:00+09:00"),
        title: String::from("test title 1"),
        memo: String::from("test memo 1"),
    };
    let task2 = todo_response::GetTask {
        id: 2,
        deadline: String::from("2021-01-11T12:34:50+09:00"),
        title: String::from("test title 2"),
        memo: String::from("test memo 2"),
    };

    // get all tasks
    let mut response = client.get(&format!("{}/api/v1/event", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::GetAllTask = response.json()?;
    assert_eq!(body.events, vec![]);

    // post a valid task
    let mut response = client
        .post(&format!("{}/api/v1/event", base_url))
        .json(&hashmap! {
            "deadline" => task1.deadline.clone(),
            "title" => task1.title.clone(),
            "memo" => task1.memo.clone(),
        })
        .send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::post_task::Success = response.json()?;
    assert_eq!(body.status, "success");
    assert_eq!(body.message, "registered");
    assert_eq!(body.id, task1.id);

    // post an invalid task
    let mut response = client
        .post(&format!("{}/api/v1/event", base_url))
        .json(&hashmap! {
            "deadline" => "invalid format",
            "title" => "test title",
            "memo" => "test memo",
        })
        .send()?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::post_task::Failure = response.json()?;
    assert_eq!(body.status, "failure");
    assert_eq!(body.message, "invalid date format");

    // get task 1
    let mut response = client.get(&format!("{}/api/v1/event/1", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::GetTask = response.json()?;
    assert_eq!(body, task1);

    // get task 2, but not found
    let response = client.get(&format!("{}/api/v1/event/2", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // post a valid task
    let mut response = client
        .post(&format!("{}/api/v1/event", base_url))
        .json(&hashmap! {
            "deadline" => task2.deadline.clone(),
            "title" => task2.title.clone(),
            "memo" => task2.memo.clone(),
        })
        .send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::post_task::Success = response.json()?;
    assert_eq!(body.status, "success");
    assert_eq!(body.message, "registered");
    assert_eq!(body.id, task2.id);

    // get task 1
    let mut response = client.get(&format!("{}/api/v1/event/1", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::GetTask = response.json()?;
    assert_eq!(body, task1);

    // get task 2
    let mut response = client.get(&format!("{}/api/v1/event/2", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::GetTask = response.json()?;
    assert_eq!(body, task2);

    // get all tasks
    let mut response = client.get(&format!("{}/api/v1/event", base_url)).send()?;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    let body: todo_response::GetAllTask = response.json()?;
    assert_eq!(body.events, vec![task1, task2]);

    assert!(server.kill().is_ok());
    Ok(())
}
