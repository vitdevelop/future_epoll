use log::{info, LevelFilter};
use crate::executor::Executor;
use crate::timer::wait;

mod executor;
mod epoll;
mod timer;
mod context;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    Executor::spawn(async {
        print_hello().await
    });

    Executor::run()?;
    Ok(())
}

async fn print_hello() {
    info!("{}", format!("Hello"));
    print_world().await
}

async fn print_world() {
    println!(" World");

    info!("Waiting");
    let seconds = 2;
    wait(seconds).await;

    info!("Elapsed {} sec", seconds);
}
