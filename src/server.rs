//! A tiny HTTP/1.1 server built on `std::net` only — no web framework.
//!
//! It understands just enough of the protocol to serve one HTML page. The point
//! is to show that Rust's standard library is enough to handle real I/O, and to
//! give you readable code to poke at.

use crate::compute::Stats;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

/// Handle a single connection: read the request line, then write the page.
pub fn handle(mut stream: TcpStream, stats: &Stats) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    // Request line looks like: "GET /path HTTP/1.1"
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let (status, body) = match path {
        "/" => ("200 OK", render_page(stats)),
        "/health" => ("200 OK", "ok".to_string()),
        _ => ("404 NOT FOUND", "<h1>404</h1>".to_string()),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Render the dashboard. We build the HTML by hand with `format!`.
fn render_page(stats: &Stats) -> String {
    // Turn the prime sample into "<li>" rows using an iterator pipeline.
    let primes = stats
        .prime_sample
        .iter()
        .map(|p| format!("<code>{p}</code>"))
        .collect::<Vec<_>>()
        .join(" · ");

    let fib = stats
        .fib
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>🦀 Rust Demo</title>
  <style>
    :root {{ color-scheme: dark; }}
    body {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      background: #0d1117; color: #e6edf3; margin: 0; padding: 2rem;
      line-height: 1.6;
    }}
    .wrap {{ max-width: 720px; margin: 0 auto; }}
    h1 {{ font-size: 2rem; }}
    .badge {{ color: #ffa657; }}
    .card {{
      background: #161b22; border: 1px solid #30363d; border-radius: 12px;
      padding: 1.25rem 1.5rem; margin: 1rem 0;
    }}
    .big {{ font-size: 2.2rem; color: #7ee787; }}
    .label {{ color: #8b949e; font-size: .85rem; text-transform: uppercase; letter-spacing: .05em; }}
    code {{ background: #21262d; padding: 1px 6px; border-radius: 6px; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; }}
    a {{ color: #58a6ff; }}
  </style>
</head>
<body>
  <div class="wrap">
    <h1>🦀 Hello from <span class="badge">Rust</span></h1>
    <p>This whole page was served by a hand-written HTTP server in
       <code>src/server.rs</code>, with zero external dependencies.</p>

    <div class="grid">
      <div class="card">
        <div class="label">Primes found below 200,000</div>
        <div class="big">{primes_found}</div>
      </div>
      <div class="card">
        <div class="label">Worker threads used</div>
        <div class="big">{workers}</div>
      </div>
      <div class="card">
        <div class="label">Largest prime</div>
        <div class="big">{largest}</div>
      </div>
      <div class="card">
        <div class="label">Computed in</div>
        <div class="big">{elapsed} ms</div>
      </div>
    </div>

    <div class="card">
      <div class="label">10 largest primes found</div>
      <p>{primes}</p>
    </div>

    <div class="card">
      <div class="label">Fibonacci (first 20)</div>
      <p>{fib}</p>
    </div>

    <p style="color:#8b949e">Edit the code in VS Code, then re-run
       <code>cargo run</code> to see it change. Try
       <a href="/health">/health</a> too.</p>
  </div>
</body>
</html>"#,
        primes_found = stats.primes_found,
        workers = stats.worker_count,
        largest = stats.largest_prime,
        elapsed = stats.elapsed_ms,
    )
}
