use std::process::{Command, ExitStatus};

fn main() {
    // build every bin
    build_project("entry_point");
    build_project("visualizer_1");
    build_project("visualizer_2");
    build_project("trainer");

    // print message indicating all builds are done
    println!("All done!");

    // copy the executable in root for our own comfort
    let res_mac = std::fs::copy("target/release/entry_point", "rustici_app");
    if res_mac.is_err() {
        let res_windows = std::fs::copy("target/release/entry_point.exe", "rustici_app.exe");
        if res_windows.is_err() {
            let _res_linux = std::fs::copy("target/release/entry_point.out", "rustici_app.out");
        }
    }
}

fn build_project(project_name: &str) {
    println!("Building project {}...", project_name);
    let status: ExitStatus = Command::new("cargo")
        .args(&["build", "--release", "--bin", project_name])
        .status()
        .expect("Failed to execute cargo build");
    if !status.success() {
        panic!("Failed to build project {}", project_name);
    }
}
