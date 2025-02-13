use dotenv::dotenv;
use kuma::{HouseState, Kuma};
use kuma_rs as kuma;
use std::env;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    let data = Kuma {
        url: env::var("URI").unwrap(),
        auth: env::var("KEY").unwrap(),
    }
    .get()
    .await
    .unwrap();

    println!("polling data...");
    println!("Status: {:?}", data.state);
    println!(
        "{}",
        match data.state {
            HouseState::Offline => "All services are down".to_owned(),
            HouseState::Degraded(a) => format!("{a} services are down"),
            HouseState::Online => "All services online".to_owned(),
        }
    );
}
