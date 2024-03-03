build:
  cargo build --bin matchbox

run: build
  sudo ./target/debug/matchbox

