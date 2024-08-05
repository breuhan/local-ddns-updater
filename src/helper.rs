use std::net::Ipv6Addr;

const _RTNLGRP_LINK: u32 = 1;
const _RTNLGRP_NEIGH: u32 = 3;
const _RTNLGRP_IPV4_IFADDR: u32 = 5;
const _RTNLGRP_IPV4_MROUTE: u32 = 6;
const _RTNLGRP_IPV4_ROUTE: u32 = 7;
const _RTNLGRP_IPV4_RULE: u32 = 8;
const RTNLGRP_IPV6_IFADDR: u32 = 9;
const _RTNLGRP_IPV6_MROUTE: u32 = 10;
const _RTNLGRP_IPV6_ROUTE: u32 = 11;
const _RTNLGRP_IPV6_RULE: u32 = 19;
const _RTNLGRP_IPV4_NETCONF: u32 = 24;
const _RTNLGRP_IPV6_NETCONF: u32 = 25;
const _RTNLGRP_MPLS_ROUTE: u32 = 27;
const _RTNLGRP_NSID: u32 = 28;
const _RTNLGRP_MPLS_NETCONF: u32 = 29;

const fn nl_mgrp(group: u32) -> u32 {
    if group > 31 {
        panic!("use netlink_sys::Socket::add_membership() for this group");
    }
    if group == 0 {
        0
    } else {
        1 << (group - 1)
    }
}

pub const fn ipv6_group() -> u32 {
    nl_mgrp(RTNLGRP_IPV6_IFADDR)
}

pub const fn is_unicast_link_local(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

pub const fn is_unique_local(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

pub const fn is_global(addr: Ipv6Addr) -> bool {
    !(addr.is_unspecified()
      || addr.is_loopback()
      // IPv4-mapped Address (`::ffff:0:0/96`)
      || matches!(addr.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
      // IPv4-IPv6 Translat. (`64:ff9b:1::/48`)
      || matches!(addr.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
      // Discard-Only Address Block (`100::/64`)
      || matches!(addr.segments(), [0x100, 0, 0, 0, _, _, _, _])
      // IETF Protocol Assignments (`2001::/23`)
      || (matches!(addr.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
          && !(
              // Port Control Protocol Anycast (`2001:1::1`)
              u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
              // Traversal Using Relays around NAT Anycast (`2001:1::2`)
              || u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
              // AMT (`2001:3::/32`)
              || matches!(addr.segments(), [0x2001, 3, _, _, _, _, _, _])
              // AS112-v6 (`2001:4:112::/48`)
              || matches!(addr.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
              // ORCHIDv2 (`2001:20::/28`)
              || matches!(addr.segments(), [0x2001, b, _, _, _, _, _, _] if b >= 0x20 && b <= 0x2F)
          )))
}

pub fn is_global_external(addr: Ipv6Addr) -> bool {
    is_global(addr) && !is_unique_local(addr) && !is_unicast_link_local(addr)
}
