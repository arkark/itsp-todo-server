extern crate assert_cmd;
use assert_cmd::prelude::*;

use std::io;
use std::process;
use std::thread;
use std::time;

fn execute_server() -> process::Child {
    process::Command::cargo_bin("itsp-todo-server")
        .unwrap()
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

#[test]
fn f() {
    let mut docker = execute_docker();
    let mut server = execute_server();

    // Sleep for 2 seconds
    thread::sleep(time::Duration::from_secs(2));

    init_database();

    assert!(kill_server(&mut server).is_ok());
    assert!(kill_docker(&mut docker)
        .map(|status| { status.success() })
        .unwrap_or(false));
}
