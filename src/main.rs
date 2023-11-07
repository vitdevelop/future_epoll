use log::{info, LevelFilter};
use crate::executor::Executor;
use crate::tcp::TcpServer;
use crate::timer::wait;

mod executor;
mod epoll;
mod timer;
mod context;
mod tcp;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    Executor::spawn(async {
        print_hello().await;
        echo().await
    });

    Executor::run()?;
    Ok(())
}

async fn echo() -> Result<()> {
    let server = TcpServer::listen("0.0.0.0:8080".to_string())?;

    info!("Enter 'telnet localhost 8080', type something and press enter");
    let client = server.accept().await?;

    let mut buff: [u8; 1024] = [0; 1024];
    client.read(buff.as_mut_slice()).await?;
    client.write(buff.as_slice()).await?;

    Ok(())
}

async fn print_hello() {
    info!("{}", format!("Hello World"));
    print_world().await
}

async fn print_world() {
    let seconds = 2;

    info!("Waiting {} secs", seconds);
    wait(seconds).await;

    info!("Elapsed {} secs", seconds);
}
