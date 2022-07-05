use std::process::{Command, Stdio};

fn main() {
    println!(env!("CARGO_MANIFEST_DIR"));
    Command::new("godot")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(format!("{}/godot", env!("CARGO_MANIFEST_DIR")))
        .output()
        .unwrap();
}
