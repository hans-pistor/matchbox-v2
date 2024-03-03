export IS_RELEASE := env_var_or_default("IS_RELEASE", "false")

build-rootfs: build
  if $IS_RELEASE; \
  then cp ./target/release/spark-server ./rootfs-builder/usr/bin/spark-server; \
  else cp ./target/debug/spark-server ./rootfs-builder/usr/bin/spark-server; \
  fi
  ./rootfs-builder/build-rootfs.sh

build:
  if $IS_RELEASE; then cargo build --release; else cargo build; fi

run: build
  if $IS_RELEASE; then sudo ./target/release/matchbox; else sudo ./target/debug/matchbox; fi

