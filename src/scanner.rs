use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;
use colored::*;
use futures::stream::FuturesUnordered;
use std::time::Duration;
use std::{
    io::ErrorKind,
    net::{Shutdown, SocketAddr, IpAddr, Ipv6Addr, Ipv4Addr},
};

/// The class for the scanner
/// Host is data type IpAddr and is the host address
/// start & end is where the port scan starts and ends
/// batch_size is how many ports at a time should be scanned
/// Timeout is the time RustScan should wait before declaring a port closed. As datatype Duration.
/// Quiet is whether or not RustScan should print things, or wait until the end to print only open ports.
/// ipv6 is whether or not this scan is an ipv6 scan.
pub struct Scanner {
    host: IpAddr,
    start: u16,
    end: u16,
    batch_size: u64,
    timeout: Duration,
    quiet: bool,
    ipv6: bool,
}

impl Scanner {
    pub fn new(
        host: IpAddr,
        start: u16,
        end: u16,
        batch_size: u64,
        timeout: Duration,
        quiet: bool,
        ipv6: bool,
    ) -> Self {
        Self {
            host: host.to_owned(),
            start,
            end,
            batch_size,
            timeout,
            quiet,
            ipv6,
        }
    }

    /// Runs scan_range with chunk sizes
    /// If you want to run RustScan normally, this is the entry point used
    /// Returns all open ports as Vec<u16>
    pub async fn run(&self) -> Vec<u16> {
        let ports: Vec<u16> = (self.start..self.end).collect();
        let mut open_ports: std::vec::Vec<u16> = Vec::new();

        for range in ports.chunks(self.batch_size as usize) {
            let mut ports = self.scan_range(range).await;
            open_ports.append(&mut ports);
        }

        open_ports
    }

    /// Given a range of ports, scan them all.
    /// Returns a vector of open ports.
    async fn scan_range(&self, range: &[u16]) -> Vec<u16> {
        let mut ftrs = FuturesUnordered::new();
            
        for port in range {
            ftrs.push(self.scan_port(*port));
        }

        let mut open_ports: Vec<u16> = Vec::new();
        while let Some(result) = ftrs.next().await {
            match result{
                Ok(port) => open_ports.push(port),
                _ => {}
            }
            
            
        }

        open_ports
    }

    /// Given a port, scan it.
    /// Turns the address into a SocketAddr
    /// Deals with the <result> type
    async fn scan_port(&self, port: u16) -> io::Result<u16> {
        let addr = SocketAddr::new(self.host, port);
        // println!("{:?}", addr);
        match self.connect(addr).await {
            Ok(x) => {
                // match stream_result.shutdown(Shutdown::Both)
                info!("Shutting down stream");
                match x.shutdown(Shutdown::Both) {
                    _ => {}
                }
                if !self.quiet {
                    println!("Open {}", port.to_string().purple());
                }
                // if connection successful
                // shut down stream
                // return port
                Ok(port)
            }
            Err(e) => match e.kind(){
                ErrorKind::Other => {
                    panic!("Too many open files. Please reduce batch size. The default is 5000. Try -b 2500.");
                }
                _ => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
            }                
            }
        

        
    
    /// Performs the connection to the socket with timeout
    async fn connect(&self, addr: SocketAddr) -> io::Result<TcpStream> {
        let stream =
            io::timeout(self.timeout, async move { TcpStream::connect(addr).await }).await?;
        info!("Returning okay from connect");
        Ok(stream)
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn does_it_run() {
        // TODO run functions here
        assert_eq!(1, 1);
    }
}
