//! Port manager: lists listening TCP ports and UDP sockets with the program
//! that owns them.

#[derive(Clone, Debug)]
pub struct PortEntry {
    pub proto: &'static str,
    pub addr: String,
    pub port: u16,
    pub pids: Vec<u32>,
}

fn finish(mut out: Vec<PortEntry>) -> Vec<PortEntry> {
    out.sort_by(|a, b| a.port.cmp(&b.port).then(a.proto.cmp(b.proto)));
    out.dedup_by(|a, b| a.port == b.port && a.proto == b.proto && a.pids == b.pids);
    out
}

/// Linux: read the kernel's socket tables in /proc directly. This needs no
/// special rights and works where the netlink route returns nothing
/// (seen on recent kernels).
#[cfg(target_os = "linux")]
pub fn listening() -> Result<Vec<PortEntry>, String> {
    let owners = linux::socket_owners();
    let mut out = Vec::new();
    let mut any_table = false;
    for (file, proto, v6) in [("tcp", "TCP", false), ("tcp6", "TCP", true), ("udp", "UDP", false), ("udp6", "UDP", true)] {
        let Ok(text) = std::fs::read_to_string(format!("/proc/net/{file}")) else { continue };
        any_table = true;
        out.extend(linux::parse_table(&text, proto, v6, &owners));
    }
    if !any_table {
        return Err("The system's network tables aren't readable".into());
    }
    Ok(finish(out))
}

#[cfg(target_os = "linux")]
pub(crate) mod linux {
    use super::PortEntry;
    use std::collections::HashMap;

    /// socket inode -> process ids, from the /proc/<pid>/fd links we may read.
    pub fn socket_owners() -> HashMap<u64, Vec<u32>> {
        let mut map: HashMap<u64, Vec<u32>> = HashMap::new();
        let Ok(procs) = std::fs::read_dir("/proc") else { return map };
        for p in procs.flatten() {
            let Some(pid) = p.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else { continue };
            let Ok(fds) = std::fs::read_dir(p.path().join("fd")) else { continue };
            for fd in fds.flatten() {
                if let Ok(target) = std::fs::read_link(fd.path()) {
                    let t = target.to_string_lossy();
                    if let Some(inode) = t.strip_prefix("socket:[").and_then(|r| r.strip_suffix(']')).and_then(|n| n.parse().ok()) {
                        let v = map.entry(inode).or_default();
                        if !v.contains(&pid) {
                            v.push(pid);
                        }
                    }
                }
            }
        }
        map
    }

    /// Decodes "0100007F:1F90" (IPv4) or 32 hex digits (IPv6); the kernel
    /// prints each 32-bit word in host (little-endian) order.
    pub fn decode_addr(hex: &str, v6: bool) -> Option<(String, u16)> {
        let (a, p) = hex.split_once(':')?;
        let port = u16::from_str_radix(p, 16).ok()?;
        let words: Vec<u32> = (0..a.len() / 8).map(|i| u32::from_str_radix(&a[i * 8..i * 8 + 8], 16)).collect::<Result<_, _>>().ok()?;
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        let addr = if v6 {
            let b: [u8; 16] = bytes.try_into().ok()?;
            std::net::Ipv6Addr::from(b).to_string()
        } else {
            let b: [u8; 4] = bytes.try_into().ok()?;
            std::net::Ipv4Addr::from(b).to_string()
        };
        Some((addr, port))
    }

    /// One /proc/net table: TCP rows in LISTEN state (0A) and every bound UDP socket.
    pub fn parse_table(text: &str, proto: &'static str, v6: bool, owners: &HashMap<u64, Vec<u32>>) -> Vec<PortEntry> {
        text.lines()
            .skip(1)
            .filter_map(|l| {
                let c: Vec<&str> = l.split_whitespace().collect();
                if c.len() < 10 {
                    return None;
                }
                if proto == "TCP" && c[3] != "0A" {
                    return None;
                }
                let (addr, port) = decode_addr(c[1], v6)?;
                if port == 0 {
                    return None;
                }
                let inode: u64 = c[9].parse().ok()?;
                Some(PortEntry { proto, addr, port, pids: owners.get(&inode).cloned().unwrap_or_default() })
            })
            .collect()
    }
}

#[cfg(not(target_os = "linux"))]
pub fn listening() -> Result<Vec<PortEntry>, String> {
    use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};
    let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets = get_sockets_info(af, proto).map_err(|e| e.to_string())?;
    let mut out: Vec<PortEntry> = Vec::new();
    for s in sockets {
        let pids = s.associated_pids.clone();
        match s.protocol_socket_info {
            ProtocolSocketInfo::Tcp(t) if t.state == TcpState::Listen => out.push(PortEntry { proto: "TCP", addr: t.local_addr.to_string(), port: t.local_port, pids }),
            ProtocolSocketInfo::Udp(u) => out.push(PortEntry { proto: "UDP", addr: u.local_addr.to_string(), port: u.local_port, pids }),
            _ => {}
        }
    }
    Ok(finish(out))
}

pub fn demo() -> Vec<PortEntry> {
    vec![
        PortEntry { proto: "TCP", addr: "127.0.0.1".into(), port: 3000, pids: vec![4242] },
        PortEntry { proto: "TCP", addr: "0.0.0.0".into(), port: 8080, pids: vec![5151] },
    ]
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    fn sees_a_port_this_test_is_listening_on() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let list = listening().expect("reading sockets works");
        let mine = list.iter().find(|e| e.port == port && e.proto == "TCP").expect("our port is listed");
        assert!(mine.pids.is_empty() || mine.pids.contains(&std::process::id()));
        assert!(list.windows(2).all(|w| w[0].port <= w[1].port), "sorted by port");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn decodes_kernel_socket_tables() {
        use super::linux::*;
        assert_eq!(decode_addr("0100007F:1F90", false), Some(("127.0.0.1".into(), 8080)));
        assert_eq!(decode_addr("00000000000000000000000001000000:0277", true), Some(("::1".into(), 631)));
        let mut owners = std::collections::HashMap::new();
        owners.insert(152458u64, vec![77u32]);
        let table = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n   0: 0100007F:B935 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 152458 1\n   1: 0100007F:0050 0200007F:9999 01 00000000:00000000 00:00000000 00000000  1000        0 5 1\n";
        let rows = parse_table(table, "TCP", false, &owners);
        assert_eq!(rows.len(), 1, "only listening sockets");
        assert_eq!((rows[0].port, rows[0].pids.clone()), (0xB935, vec![77]));
    }

    #[test]
    fn demo_rows() {
        assert_eq!(demo().len(), 2);
    }
}
