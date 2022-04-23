use tracing::instrument;
use tracing_subscriber::prelude::*;

#[instrument]
fn fibonacci(n: usize) -> usize {
    if n < 2 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[instrument]
fn fibonacci_parallel(n: usize) -> usize {
    if n < 20 {
        fibonacci(n - 1) + fibonacci(n - 2)
    } else {
        let (a, b) = rayon::join(|| fibonacci_parallel(n - 1), || fibonacci_parallel(n - 2));
        a + b
    }
}

fn main() {
    let _guard = if std::env::args().any(|arg| arg == "--no-trace") {
        None
    } else {
        let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(true)
            .build();
        tracing_subscriber::registry().with(chrome_layer).init();
        Some(guard)
    };

    let before = std::time::Instant::now();
    println!("fibonacci_serial(28) -> {}", fibonacci(28));
    println!("took {} s", before.elapsed().as_secs_f32());
    let before = std::time::Instant::now();
    println!("fibonacci_parallel(28) -> {}", fibonacci_parallel(28));
    println!("took {} s", before.elapsed().as_secs_f32());
}
