To connect a network namespace to WAN, you need to do the following

1. Create a network namespace
```bash
ip netns add netns-name
```

2. Create the veth pair

```bash
ip link add fc-veth type veth peer name fc-peer
```

3. Assign an IP address to the veth device and activate it
```bash
ip address add 10.200.1.1/24 dev veth
ip link set veth up
```

4. Move the peer into the network namespace, assign it an IP address, and
   activate it

```bash
ip link set peer netns netns-name
ip netns exec netns-name ip address add 10.200.1.2/24 dev peer
ip netns exec netns-name ip link set peer up
ip netns exec netns-name ip link set lo up
```

5. Add a default route for the network namespace via the address of veth device
   (in the root namespace)

```bash
ip netns exec netns-name ip route add default via 10.200.1.1
```

6. Enable MASQUERADE for packets coming from the PEER address, going to the host
   interface (ens4)

```bash
iptables -t nat -A POSTROUTING -s 10.200.1.2/24 -o ens4 -j MASQUERADE
```

7. Enable forwarding packets from veth to ens4 & vice versa
```bash
iptables -A FORWARD -i ens4 -o veth -j ACCEPT
iptables -A FORWARD -o ens4 -i veth -j ACCEPT
```
