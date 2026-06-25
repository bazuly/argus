use crate::collectors::models::Protocol;
use crate::model::PortBinding;
use anyhow::Result;
use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};
use sysinfo::{Pid, ProcessesToUpdate, System};

pub fn collect(port_filter: Option<u16>) -> Result<Vec<PortBinding>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    )?;

    // structures storage
    let mut bindings: Vec<PortBinding> = Vec::new();

    for socket in sockets {
        // select first pid from socket
        let pid: Option<u32> = socket.associated_pids.first().copied();

        match socket.protocol_socket_info {
            // --- TCP ---
            ProtocolSocketInfo::Tcp(tcp_info) => {
                // only LISTEN sockets
                if tcp_info.state != TcpState::Listen {
                    continue;
                }
                if let Some(wanted_port) = port_filter {
                    if tcp_info.local_port != wanted_port {
                        continue;
                    }
                }
                bindings.push(PortBinding {
                    port: tcp_info.local_port,
                    protocol: Protocol::Tcp,
                    address: tcp_info.local_addr.to_string(),
                    pid,
                    process_name: None,
                });
            }

            ProtocolSocketInfo::Udp(udp_info) => {
                // udp do not have state, if port is open - it in list
                if let Some(wanted_port) = port_filter {
                    if udp_info.local_port != wanted_port {
                        continue;
                    }
                }
                bindings.push(PortBinding {
                    port: udp_info.local_port,
                    protocol: Protocol::Udp,
                    address: udp_info.local_addr.to_string(),
                    pid,
                    process_name: None,
                });
            }
        }
    }

    enrich_with_process_names(&mut bindings);

    // sort, first - port, second - address
    bindings.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then(left.address.cmp(&right.address))
    });
    Ok(bindings)
}

// for each PortBinding with known PID retrieve process name
fn enrich_with_process_names(bindings: &mut [PortBinding]) {
    let pids: Vec<Pid> = bindings
        .iter()
        .filter_map(|binding| binding.pid)
        .map(Pid::from_u32)
        .collect();

    if pids.is_empty() {
        return;
    }

    let mut system = System::new();

    // update in sysinfo only necessary ports
    // not all from the whole operating system

    system.refresh_processes(ProcessesToUpdate::Some(&pids), true);

    for binding in bindings.iter_mut() {
        let Some(raw_pid) = binding.pid else {
            continue;
        };
        let pid = Pid::from_u32(raw_pid);

        if let Some(process) = system.process(pid) {
            let name = process.name().to_string_lossy().into_owned();
            binding.process_name = Some(name);
        }
    }
}
