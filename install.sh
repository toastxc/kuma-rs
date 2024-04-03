mold --run cargo b -r --example gui
cp ./target/release/examples/gui ./target/release/
cargo-pak generate
cargo-pak build
cargo-pak install