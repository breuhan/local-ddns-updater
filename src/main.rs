use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

use netlink_packet_core::NetlinkPayload;
use netlink_packet_route::address::AddressAttribute::Address;
use netlink_packet_route::RouteNetlinkMessage;
use netlink_sys::{AsyncSocket, SocketAddr};
use rtnetlink::{new_connection, Handle};
use std::net::IpAddr;
use std::net::Ipv6Addr;
use tokio::time::timeout;

mod helper;
use helper::*;

use clap::Parser;

/// Global IPv6 address change listener, which executes scripts in a specified directory on IPv6 address change.
/// This program listens for IPv6 address changes on a specified interface and executes scripts in a specified directory.
/// The purpose of this program is update internal DNS records on IPv6 address change.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Interface name to listen for IPv6 address changes
    #[arg(short, long)]
    interface: String,
    /// plugin folder, where executable scripts are executed on IPv6 address change
    #[arg(short, long, default_value_t = String::from("/etc/local-ddns-updater/"))]
    dir: String,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    let link_name = args.interface.as_str();
    let dir_path = Path::new(&args.dir);
    let verbose = args.verbose;

    let (mut conn, handle, mut messages) = new_connection().map_err(|e| format!("{e}"))?;

    // Bind to the IPv6 address group.
    let addr = SocketAddr::new(0, ipv6_group());
    conn.socket_mut()
        .socket_mut()
        .bind(&addr)
        .expect("Failed to bind");

    // Spawn `Connection` to start polling netlink socket.
    tokio::spawn(conn);

    if get_link_by_name(&handle, link_name).await.is_none() {
        eprintln!("No interface with {link_name} found.");
        std::process::exit(2);
    }

    let mut run = Run::new(dir_path.to_path_buf(), verbose);

    // Start receiving events through `messages` channel.
    while let Some((message, _)) = messages.next().await {
        let payload = message.payload;

        if let NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewAddress(new_address)) = payload
        {
            if new_address.header.index
                == get_link_by_name(&handle, link_name)
                    .await
                    .expect("Interface lost")
            {
                for attrib in new_address.attributes {
                    if let Address(IpAddr::V6(ipv6)) = attrib {
                        run.update(ipv6).await;
                    }
                }
            }
        }
    }
    Ok(())
}

struct Run {
    dir_path: PathBuf,
    verbose: bool,
    current_addr: Option<Ipv6Addr>,
}

impl Run {
    fn new(dir_path: PathBuf, verbose: bool) -> Self {
        Self {
            dir_path,
            verbose,
            current_addr: None,
        }
    }

    async fn update(&mut self, ipv6: Ipv6Addr) {
        if self.needs_update(ipv6) && is_global_external(ipv6) {
            self.current_addr = Some(ipv6);
            if self.verbose {
                println!("IPv6 address changed to: {:?}", ipv6);
            }
            self.execute_scripts().await;
        }
    }

    fn needs_update(&self, addr: Ipv6Addr) -> bool {
        match self.current_addr {
            Some(current) => current != addr,
            None => true,
        }
    }

    async fn execute_scripts(&self) {
        if let Some(addr) = self.current_addr {
            self.execute_scripts_for_addr(addr).await;
        }
    }

    async fn execute_scripts_for_addr(&self, addr: Ipv6Addr) {
        // Iterate over the directory entries
        match fs::read_dir(&self.dir_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let script = entry.path();
                    if script.is_file() {
                        if let Ok(metadata) = fs::metadata(&script) {
                            if metadata.permissions().mode() & 0o111 != 0 {
                                if self.verbose {
                                    println!("Executing file: {:?}", script);
                                }
                                let timelimit = tokio::time::Duration::from_secs(10);
                                // Spawn a new thread to run the command asynchronously
                                match tokio::spawn(timeout(timelimit, execute_script(script.clone(), addr)))
                                    .await
                                {
                                    Ok(_) => (),
                                    Err(e) => eprintln!("Error executing script: {:?}", e),
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading scripts directory: {}", e.to_string()),
        }
    }
}

async fn execute_script(script: PathBuf, addr: Ipv6Addr) -> Result<(), std::io::Error> {
    Command::new(script).args([addr.to_string()]).status()?;
    Ok(())
}

async fn get_link_by_name(handle: &Handle, name: &str) -> Option<u32> {
    let mut links = handle.link().get().match_name(name.to_string()).execute();
    let link_opt = links.try_next().await.ok()?;

    if let Some(link) = link_opt {
        Some(link.header.index)
    } else {
        None
    }
}
