use kuma_rs as kuma;


use std::env;
use dotenv::dotenv;
use kuma::api::{HouseState, Kuma};

#[tokio::main]
async fn main() {
    let _ = dotenv();
    let data = Kuma::new(env::var("URI").unwrap(), env::var("KEY").unwrap())
        .get()
        .await
        .unwrap();


    println!("polling data...");
    println!("Status: {:?}", data.state);
    println!("{}", match data.state {
        HouseState::Offline => "All services are down".to_owned(),
        HouseState::Degraded(a) => format!("{a} services are down"),
        HouseState::Online => "All services online".to_owned(),
    });
}
