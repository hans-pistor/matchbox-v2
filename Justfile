export IS_RELEASE := env_var_or_default("IS_RELEASE", "false")

build-rootfs: build
  if $IS_RELEASE; \
  then cp ./target/x86_64-unknown-linux-musl/release/spark-server ./rootfs-builder/usr/bin/spark-server; \
  else cp ./target/x86_64-unknown-linux-musl/debug/spark-server ./rootfs-builder/usr/bin/spark-server; \
  fi
  chmod +x ./rootfs-builder/usr/bin/spark-server
  ./rootfs-builder/build-rootfs.sh

run-rootfs: 
  docker run -it $(docker build -q ./rootfs-builder) sh

build:
  if $IS_RELEASE; then cargo build --release; else cargo build; fi

run: host-networking-setup build
  if $IS_RELEASE; then sudo ./target/x86_64-unknown-linux-musl/release/matchbox; else sudo ./target/x86_64-unknown-linux-musl/debug/matchbox; fi

host-networking-setup:
  # Enable ipv4 forwarding
  sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"
  
test:
  cargo t -- --nocapture
  cargo t -- --ignored --nocapture

create-sandbox:
  curl --header "Content-Type: application/json" --request POST --data '{"code_drive_path": {"type": "Local", "path": "/tmp/code-drive.img"}}' http://localhost:3000/sandbox

execute-sandbox SANDBOX_ID:
  curl --request POST http://localhost:3000/sandbox/{{SANDBOX_ID}}/execute