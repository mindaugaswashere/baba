//! The "does some Rust stuff" part: parallel number crunching.
//!
//! We split work across real OS threads and collect the results over a channel.
//! This is the classic example of Rust's "fearless concurrency" — the compiler
//! guarantees at build time that we never share data unsafely between threads.

use std::sync::mpsc;
use std::thread;
use std::time::Instant;

/// Everything we compute at startup, later rendered into the web page.
pub struct Stats {
    pub primes_found: usize,
    pub largest_prime: u64,
    pub prime_sample: Vec<u64>,
    pub fib: Vec<u64>,
    pub worker_count: usize,
    pub elapsed_ms: u128,
}

/// Run the whole demo: fan out prime-finding across threads, plus a Fibonacci
/// sequence on the main thread, and time the whole thing.
pub fn run_demo() -> Stats {
    let start = Instant::now();

    let worker_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let upper = 200_000u64;

    let primes = parallel_primes(upper, worker_count);

    Stats {
        primes_found: primes.len(),
        largest_prime: primes.last().copied().unwrap_or(0),
        // `.rev().take(10)` + collect: iterators are lazy and compose with zero cost.
        prime_sample: primes.iter().rev().take(10).copied().collect(),
        fib: fibonacci(20),
        worker_count,
        elapsed_ms: start.elapsed().as_millis(),
    }
}

/// Find every prime up to `upper`, splitting the range across `workers` threads.
fn parallel_primes(upper: u64, workers: usize) -> Vec<u64> {
    let chunk = upper / workers as u64;
    let (tx, rx) = mpsc::channel();

    // Each thread takes ownership of its own sub-range and a clone of the sender.
    for w in 0..workers as u64 {
        let tx = tx.clone();
        let lo = w * chunk + 1;
        let hi = if w == workers as u64 - 1 { upper } else { (w + 1) * chunk };
        thread::spawn(move || {
            let found: Vec<u64> = (lo..=hi).filter(|&n| is_prime(n)).collect();
            // If the receiver is gone we simply stop — hence `let _ =`.
            let _ = tx.send(found);
        });
    }
    // Drop our original sender so the channel closes once all workers finish.
    drop(tx);

    // Collect every chunk, then sort so the merged result is in order.
    let mut all: Vec<u64> = rx.iter().flatten().collect();
    all.sort_unstable();
    all
}

/// Trial division — small and readable rather than the fastest possible.
fn is_prime(n: u64) -> bool {
    match n {
        0 | 1 => false,
        2 | 3 => true,
        _ if n % 2 == 0 => false,
        _ => {
            let mut i = 3;
            while i * i <= n {
                if n % i == 0 {
                    return false;
                }
                i += 2;
            }
            true
        }
    }
}

/// Build the first `n` Fibonacci numbers iteratively (no recursion, no overflow
/// for n <= ~93 since we use u64).
fn fibonacci(n: usize) -> Vec<u64> {
    let mut seq = Vec::with_capacity(n);
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        seq.push(a);
        (a, b) = (b, a + b);
    }
    seq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primes_basic() {
        assert!(is_prime(2));
        assert!(is_prime(97));
        assert!(!is_prime(1));
        assert!(!is_prime(100));
    }

    #[test]
    fn fib_starts_right() {
        assert_eq!(fibonacci(7), vec![0, 1, 1, 2, 3, 5, 8]);
    }

    #[test]
    fn parallel_matches_serial() {
        let serial: Vec<u64> = (1..=1000).filter(|&n| is_prime(n)).collect();
        let parallel = parallel_primes(1000, 4);
        assert_eq!(serial, parallel);
    }
}
