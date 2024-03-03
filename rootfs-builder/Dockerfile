FROM alpine

WORKDIR /root

# Set root password
RUN echo "root:root" | chpasswd

# Download dependencies
RUN apk add --update --no-cache openrc util-linux openssh rng-tools vim

RUN ssh-keygen -A

# Setup login terminal on ttyS0
RUN ln -s agetty /etc/init.d/agetty.ttyS0 \
    && echo ttyS0 > /etc/securetty \
    && rc-update add agetty.ttyS0 default

# Make sure special file systems are mounted on boot
RUN rc-update add devfs boot \
    && rc-update add procfs boot \
    && rc-update add sysfs boot \
    && rc-update add local default


COPY etc /etc
COPY root /root

# Need to start rngd before sshd otherwise VM startup will hang
# This still takes ~5s so there's room for improvement
RUN rc-update add rngd \ 
    && rc-update add firecracker-networking \
    && rc-update add sshd