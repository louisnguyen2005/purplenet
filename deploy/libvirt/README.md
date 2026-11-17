# Proposed libvirt lab network

The selected first-lab target is `10.77.0.0/24` on a libvirt-managed network named `purplenet-isolated`. The XML has **no `<forward>` element**, so libvirt will not route guest traffic to other LANs through this virtual network. Guests can still communicate with the host's `10.77.0.1` bridge address. Host firewall and guest routing must be reviewed separately. A guest attached to another network, or acting as a router, can defeat this isolation assumption. See the [libvirt network format](https://libvirt.org/formatnetwork.html).

| Address range | Intended use |
|---|---|
| `10.77.0.1` | libvirt host bridge; excluded from test scope |
| `10.77.0.10–10.77.0.49` | proposed static guest addresses; assign only after inventory review |
| `10.77.0.100–10.77.0.199` | libvirt DHCP pool |

The included `scope-template.json` is deliberately expired and contains an owner placeholder. It cannot authorize a run. Replace the owner, enumerate the actual guest asset IDs/IPs, set a bounded window, review exclusions, and obtain a distinct human approval before producing a live scope. The current CLI has only a mock runner; it does not connect to this network.

## Deployment sequence when libvirt is available

1. Confirm `10.77.0.0/24` does not overlap host routes, VPNs, containers, or another libvirt network.
2. Review the XML and host firewall exposure. Confirm guests have **only** the intended isolated interface.
3. Define the network with `virsh net-define deploy/libvirt/purplenet-isolated.xml`.
4. Start it deliberately with `virsh net-start purplenet-isolated`; do not enable autostart until the owner approves that choice.
5. Inspect `virsh net-dumpxml purplenet-isolated` and `virsh net-list --all`, then document assigned guest IPs and the snapshot/restore owner.
6. Before any exercise, confirm the guest has no second route to a production or Internet-connected network.

These commands are documentation only. They have not been run on this host. The `virsh` executable was not found during project setup. Libvirt's [virsh reference](https://www.libvirt.org/manpages/virsh.html) documents the network commands.
