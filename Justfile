export IS_RELEASE := env_var_or_default("IS_RELEASE", "false")

build-rootfs: build
  if $IS_RELEASE; \
  then cp ./target/release/spark-server ./rootfs-builder/usr/bin/spark-server; \
  else cp ./target/debug/spark-server ./rootfs-builder/usr/bin/spark-server; \
  fi
  ./rootfs-builder/build-rootfs.sh

run-rootfs: 
  docker run -it $(docker build -q ./rootfs-builder) sh

build:
  if $IS_RELEASE; then cargo build --release; else cargo build; fi

run: host-networking-setup build
  if $IS_RELEASE; then sudo ./target/release/matchbox; else sudo ./target/debug/matchbox; fi

host-networking-setup:
  # Enable ipv4 forwarding
  sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"

  # Delete MASQUERADE rule if it exists
  sudo iptables -t nat -D POSTROUTING -o ens4 -j MASQUERADE || true
  sudo iptables -t nat -A POSTROUTING -o ens4 -j MASQUERADE

  # Delete conntrac forwarding rule if it exists
  sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT || true
  sudo iptables -I FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT