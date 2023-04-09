use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::io::{Read, Write};

fn main() {
    let mut server = Command::new("./startserver.sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    let mut server_stdin = server.stdin.take().expect("Failed to get server stdin");
    let mut server_stdout = server.stdout.take().expect("Failed to get server stdout");
    let mut server_stderr = server.stderr.take().expect("Failed to get server stderr");

    let stop_requested = Arc::new(AtomicBool::new(false));
    let stop_sent = Arc::new(AtomicBool::new(false));
    let server_arc = Arc::new(Mutex::new(server));

    let stop_requested_handler = stop_requested.clone();
    let stop_sent_clone = stop_sent.clone();
    let server_arc_clone = server_arc.clone();

    // Handle SIGTERM signal to stop server gracefully
    ctrlc::set_handler(move || {
        println!("[WRAPPER INFO]: SIGTERM received ! Handling the stop request…");
        stop_requested_handler.store(true, Ordering::SeqCst);
    })
    .expect("Failed to register SIGTERM handler");

    let _stdout_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match server_stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let stdout_str = std::str::from_utf8(&buffer[..n]).unwrap();
                    print!("{}", stdout_str);
                    if stdout_str.contains("Restarting automatically") {
                        // Send a SIGINT signal to avoid server restart
                        println!("[WRAPPER INFO]: Server waiting for automatic restart, sending SIGINT to avoid it…");
                        server_arc_clone.lock().unwrap().kill().expect("Failed to send SIGINT signal to server");
                        stop_sent_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read server stdout: {}", e);
                    break;
                }
            }
        }
    });

    let _stderr_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match server_stderr.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let stderr_str = std::str::from_utf8(&buffer[..n]).unwrap();
                    eprint!("{}", stderr_str);
                }
                Err(e) => {
                    eprintln!("Failed to read server stderr: {}", e);
                    break;
                }
            }
        }
    });

    while !stop_requested.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("[WRAPPER INFO]: Sending a \"stop\" command to the server");
    server_stdin
        .write_all(b"stop\n")
        .expect("Failed to send 'stop' command to server");

    while !stop_sent.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    println!("[WRAPPER INFO]: Waiting for the server to terminate…");
    server_arc.lock().unwrap().wait()
        .expect("Failed to wait for server process to exit");

    println!("[WRAPPER INFO]: Server stopped gracefully, exiting wrapper…")
}
