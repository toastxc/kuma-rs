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

## GUI
<h1 align="center">
<img src="https://github.com/toastxc/kuma-rs/blob/main/README_RESOURCES/img1.png" alt="Demo image 1" width="30%" height="30%">
<img src="https://github.com/toastxc/kuma-rs/blob/main/README_RESOURCES/img2.png" alt="Demo image 2" width="30%" height="30%">
</h1>


### Installing
You'll need a few flatpak dependencies to install
```bash
# clone the repository
git clone https://github.com/toastxc/kuma-rs.git
# cargo-pak manages flatpaks for rust apps, its safe dw
cargo install cargo-pak
cd kuma-rs/
# this uses cargo-pak
sh install.sh
flatpak run xyz.toastxc.Kuma
```