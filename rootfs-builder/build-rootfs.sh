set -eu
DOCKER_DIRECTORY=$(dirname "$0")
ROOTFS_DIR=/tmp-rootfs
HOST_ROOTFS_DIR=/tmp$ROOTFS_DIR
ROOTFS_FILE=/tmp/rootfs.ext4
DOCKER_TAG='rootfs-builder'


rm -f $ROOTFS_FILE
sudo umount $HOST_ROOTFS_DIR || true
sudo rm -rf $HOST_ROOTFS_DIR

# Create file
dd if=/dev/zero of=$ROOTFS_FILE bs=1M count=1024

# Create empty filesystem
sudo mkfs.ext4 $ROOTFS_FILE

# Make sure directory is created
mkdir -p $HOST_ROOTFS_DIR

# Mount the filesystem 
sudo mount $ROOTFS_FILE $HOST_ROOTFS_DIR


# Build our custom rootfs builder
docker build --tag $DOCKER_TAG $DOCKER_DIRECTORY

docker run -it --rm -v $HOST_ROOTFS_DIR:$ROOTFS_DIR $DOCKER_TAG sh copy-to-rootfs $ROOTFS_DIR

sudo umount $HOST_ROOTFS_DIR