//! Rust capability demo — a zero-dependency web dashboard.
//!
//! This single binary:
//!   1. Spawns a pool of worker threads that crunch numbers in parallel.
//!   2. Runs a tiny multi-threaded HTTP server (std library only).
//!   3. Opens your browser to a live dashboard.
//!
//! It's deliberately written to show off several Rust ideas in a small space:
//! threads + channels (`mod compute`), ownership/borrowing, pattern matching,
//! iterators, and a hand-rolled HTTP server (`mod server`).

mod compute;
mod server;

use std::net::TcpListener;
use std::process::Command;
use std::sync::Arc;
use std::thread;

fn main() {
    let port = 7878;
    let addr = format!("127.0.0.1:{port}");

    // Run the parallel computations once at startup. `Arc` lets us share the
    // result across every request handler thread without copying it.
    let stats = Arc::new(compute::run_demo());

    let listener = TcpListener::bind(&addr).expect("could not bind to port");
    println!("\n  🦀 Rust demo running at http://{addr}");
    println!("  Press Ctrl+C to stop.\n");

    // Best-effort: pop open the browser. Failure here is fine — the URL is printed.
    open_browser(&format!("http://{addr}"));

    // Accept connections forever. Each gets its own thread; `Arc::clone` is cheap
    // (just bumps a reference count), so every thread shares the same stats.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stats = Arc::clone(&stats);
                thread::spawn(move || server::handle(stream, &stats));
            }
            Err(e) => eprintln!("connection failed: {e}"),
        }
    }
}

/// Tries the platform-appropriate "open this URL" command. WSL, Linux, macOS.
fn open_browser(url: &str) {
    // On WSL, `wslview` or `explorer.exe` reach the Windows browser.
    let candidates = ["wslview", "explorer.exe", "xdg-open", "open"];
    for cmd in candidates {
        if Command::new(cmd).arg(url).spawn().is_ok() {
            return;
        }
    }
    println!("  (Couldn't auto-open a browser — just visit the URL above.)");
}
