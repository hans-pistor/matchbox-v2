build-rootfs:
  ./rootfs-builder/build-rootfs.sh

build:
  cargo build --bin matchbox

run: build
  sudo ./target/debug/matchbox

