use std::io::Seek;
use crate::executor::Executor;

mod executor;
mod task;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

fn main() {
    let executor = Executor::new();

    executor.spawn(async {
        print_hello().await
    });

    executor.run();
}

async fn print_hello() {
    print!("Hello");
    print_world().await
}

async fn print_world() {
    println!(" World")
}
