cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu

mv target/aarch64-apple-darwin/release/yazi target/yazi-aarch64-apple-darwin
mv target/x86_64-apple-darwin/release/yazi target/yazi-x86_64-apple-darwin
mv target/x86_64-unknown-linux-gnu/release/yazi target/yazi-x86_64-unknown-linux-gnu

zip -j yazi-aarch64-apple-darwin.zip target/yazi-aarch64-apple-darwin
zip -j yazi-x86_64-apple-darwin.zip target/yazi-x86_64-apple-darwin
zip -j yazi-x86_64-unknown-linux-gnu.zip target/yazi-x86_64-unknown-linux-gnu
