# Project Name

The "local-ddns-updater" is a tool to update local DNS records for IPv6 network.

## Description

I have an internal DNS server that I use to access some internal/external services.
The reason for writing this program is that I want to be able to access my servers with the same internal/external IPv6 address.
My ISP provides me with a dynamic IPv6 prefix, this means that the prefix can change at any time.  With stateless address autoconfiguration (SLAAC) there is no other way to update the DNS records automatically.

The "local-ddns-updater" tool solves this problem by subscriping to the Linux kernel's netlink interface and listening for changes in the IPv6 address of the network interface.
When the address changes, the tool will just execute a set of scripts that can be used to update the DNS records. Or whatever you want to do with the new address.

## Features

- Automatic IP address detection
- No polling required
- Customizable update scripts

## Installation

To install the "local-ddns-updater" tool, follow these steps:

### Manual installation

```bash
cargo build --release
sudo cp target/release/local-ddns-updater /usr/local/bin/
sudo cp local-ddns-updater@.service /etc/systemd/system/
```

### Debian/Ubuntu

```bash
sudo dpkg -i local-ddns-updater.deb
```

## Usage

To enable the service:

```bash
sudo systemctl enable --now local-ddns-updater@eth0.service
```

## Contributing

Contributions are welcome! If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
