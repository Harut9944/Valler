//! Port manager: lists listening TCP ports and UDP sockets with the program
//! that owns them.

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};

#[derive(Clone, Debug)]
pub struct PortEntry {
    pub proto: &'static str,
    pub addr: String,
    pub port: u16,
    pub pids: Vec<u32>,
}

pub fn listening() -> Result<Vec<PortEntry>, String> {
    let af = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets = get_sockets_info(af, proto).map_err(|e| e.to_string())?;
    let mut out: Vec<PortEntry> = Vec::new();
    for s in sockets {
        let pids = s.associated_pids.clone();
        match s.protocol_socket_info {
            ProtocolSocketInfo::Tcp(t) if t.state == TcpState::Listen => out.push(PortEntry {
                proto: "TCP",
                addr: t.local_addr.to_string(),
                port: t.local_port,
                pids,
            }),
            ProtocolSocketInfo::Udp(u) => out.push(PortEntry {
                proto: "UDP",
                addr: u.local_addr.to_string(),
                port: u.local_port,
                pids,
            }),
            _ => {}
        }
    }
    out.sort_by(|a, b| a.port.cmp(&b.port).then(a.proto.cmp(b.proto)));
    out.dedup_by(|a, b| a.port == b.port && a.proto == b.proto && a.pids == b.pids);
    Ok(out)
}

pub fn demo() -> Vec<PortEntry> {
    vec![
        PortEntry { proto: "TCP", addr: "127.0.0.1".into(), port: 3000, pids: vec![4242] },
        PortEntry { proto: "TCP", addr: "0.0.0.0".into(), port: 8080, pids: vec![5151] },
    ]
}
