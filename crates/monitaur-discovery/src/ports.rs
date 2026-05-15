use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use monitaur_core::error::EngineResult;
use monitaur_core::models::Port;
use monitaur_core::models::Protocol;

const COMMON_PORTS: &[u16] = &[
    22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995, 1433, 1521, 2049, 2181, 2375,
    2376, 2377, 3000, 3306, 3389, 4000, 4222, 5000, 5432, 5601, 6379, 6443, 7443, 8000, 8001, 8080,
    8443, 8888, 9000, 9042, 9092, 9100, 9200, 9300, 9419, 9999, 11211, 15672, 27017, 27018, 27019,
];

#[derive(Default)]
pub struct PortScanner;

impl PortScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_common_ports(&self, targets: &[String]) -> EngineResult<Vec<(String, Vec<Port>)>> {
        let mut results = Vec::new();

        for target in targets {
            let mut open_ports = Vec::new();
            for &port in COMMON_PORTS {
                let addr: SocketAddr = match format!("{target}:{port}").parse() {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
                    open_ports.push(Port {
                        port,
                        protocol: Protocol::Tcp,
                        exposed: true,
                    });
                }
            }
            results.push((target.clone(), open_ports));
        }

        Ok(results)
    }
}
