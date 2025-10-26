cargo build --release
cp target/release/linkifier /opt/linkifier/bin/linkifier
systemctl restart linkifier.service
