cargo +stable build --release --target aarch64-apple-darwin
cargo +stable build --release --target x86_64-apple-darwin
cargo +stable build --release --target x86_64-unknown-linux-gnu

mv target/aarch64-apple-darwin/release/yazi yazi-aarch64-apple-darwin
mv target/x86_64-apple-darwin/release/yazi yazi-x86_64-apple-darwin
mv target/x86_64-unknown-linux-gnu/release/yazi yazi-x86_64-unknown-linux-gnu

zip -j yazi-aarch64-apple-darwin.zip yazi-aarch64-apple-darwin
zip -j yazi-x86_64-apple-darwin.zip yazi-x86_64-apple-darwin
zip -j yazi-x86_64-unknown-linux-gnu.zip yazi-x86_64-unknown-linux-gnu
