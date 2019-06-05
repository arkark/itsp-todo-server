extern crate assert_cmd;
use assert_cmd::prelude::*;

use std::process;

fn execute_server() -> process::Child {
    process::Command::cargo_bin("itsp-todo-server")
        .unwrap()
        .spawn()
        .expect("Failed to execute itsp-todo-server")
}

fn kill_server(server: &mut process::Child) {
    server.kill().expect("Failed to kill itsp-todo-server");
}

fn execute_docker() -> process::Child {
    process::Command::new("docker-compose")
        .current_dir("./tests")
        .arg("up")
        .spawn()
        .expect("Failed to execute `docker-compose up`")
}

fn kill_docker(docker: &mut process::Child) {
    docker
        .kill()
        .map(|_| {
            process::Command::new("docker-compose")
                .arg("down")
                .output()
                .expect("Failed to execute `docker-compose down`");
        })
        .expect("Failed to kill docker-compose");
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

#[test]
fn f() {
    let mut docker = execute_docker();
    let mut server = execute_server();

    // Sleep for 2 seconds
    std::thread::sleep(std::time::Duration::from_secs(2));

    init_database();

    kill_server(&mut server);
    kill_docker(&mut docker);
}
