extern crate assert_cmd;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;

use assert_cmd::prelude::*;
use reqwest::{header, StatusCode};

use std::io;
use std::process;
use std::thread;
use std::time;

const SERVER_PORT: &str = "8888";

pub mod todo_response {
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct GetTask {
        pub id: i64,
        pub deadline: String,
        pub title: String,
        pub memo: String,
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
            "postgresql://username:password@localhost:5432/itsp-todo-app",
        )
        .env("SERVER_PORT", SERVER_PORT)
        .spawn()
        .expect("Failed to execute itsp-todo-server")
}

fn kill_server(server: &mut process::Child) -> io::Result<()> {
    server.kill()
}

fn execute_docker() -> process::Child {
    process::Command::new("docker-compose")
        .current_dir("./tests")
        .arg("up")
        .spawn()
        .expect("Failed to execute `docker-compose up`")
}

fn kill_docker(docker: &mut process::Child) -> io::Result<process::ExitStatus> {
    docker.kill().map(|_| {
        process::Command::new("docker-compose")
            .current_dir("./tests")
            .arg("down")
            .output()
            .expect("Failed to execute `docker-compose down`")
            .status
    })
}

fn init_database() {
    process::Command::new("diesel")
        .arg("setup")
        .output()
        .expect("Failed to execute `diesel setup`");
    process::Command::new("diesel")
        .arg("migration")
        .arg("redo")
        .output()
        .expect("Failed to execute `diesel migration redo`");
}

fn initialize() -> (process::Child, process::Child) {
    let docker = execute_docker();
    let server = execute_server();
    thread::sleep(time::Duration::from_secs(5));
    init_database();

    // Sleep for 5 seconds
    thread::sleep(time::Duration::from_secs(5));

    (server, docker)
}

fn terminate(server: &mut process::Child, docker: &mut process::Child) {
    assert!(kill_server(server).is_ok());
    assert!(kill_docker(docker)
        .map(|status| { status.success() })
        .unwrap_or(false));
}

#[test]
fn server_api() -> Result<(), reqwest::Error> {
    let (mut server, mut docker) = initialize();

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

    terminate(&mut server, &mut docker);
    Ok(())
}
