#!/bin/bash
set -exuo pipefail
IP=192.168.0.1
cargo build --release --target x86_64-unknown-linux-musl
ssh root@$IP systemctl stop cthulhu
scp target/x86_64-unknown-linux-musl/release/cthulhu root@$IP:/home/root/cthulhu/
ssh root@$IP systemctl start cthulhu
