## A library for UptimeKuma, at last
If you use both Rust and Kuma you may have noticed that there practically are no libraries for it, until now

## Purpose
I made this program as part of a project integrate LEDs with Kuma

## Usage
There is no binary of CLI yet but you can use the library, all you need is a few dependencies, a Kuma instance and an .env file
### dotenv
```dotenv
# the key will look something like this
# do NOT include a ':' at the start, it is not needed
# don't share your key with anyone
KEY="OFND4SB6IFBD8S9FDFO9JDS4FF2DS7ydhsuifdsdfd"
# the URI should NOT include HTTP/HTTPS
# HTTP support is not currently included
URI="kuma.instance/metrics"
```
### main.rs
```rust
#[tokio::main]
async fn main() {
    let _ = dotenv();
    let data = Kuma::new(env::var("URI").unwrap(), env::var("KEY").unwrap())
    .get()
    .await
    .unwrap();
}
```
