use std::{net::{IpAddr, Ipv4Addr, UdpSocket}, collections::HashMap, time::Duration, io::ErrorKind};
use shared::MULTICAST_PORT;

use ratatui::{prelude::*, Frame, widgets::{ListItem, List, Borders, Block}};

pub struct FindState {
    pub ip: Option<IpAddr>,
    found: HashMap<String, IpAddr>,
    socket: UdpSocket,
    multicast_addr: Ipv4Addr
}

impl FindState {
    pub fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
            .split(f.size());
        
        let title = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().dark_gray())
            .title(format!("Looking for device with name '{}' on multicast {:?}:{:?}", shared::NAME, self.multicast_addr, shared::MULTICAST_PORT))
            .title_alignment(Alignment::Center)
            .title_style(Style::new().reset());

        f.render_widget(title, chunks[0]);

        let hosts: Vec<ListItem> = self
            .found
            .iter()
            .map(|(name, addr)| {
                ListItem::new(vec![
                    Line::from(format!("{:#?} ({:?})", name.trim(), addr)),
                ])
            })
            .collect();

        let hosts_list = List::new(hosts)
            .block(Block::default().borders(Borders::ALL).title("Found Devices"))
            .start_corner(Corner::BottomLeft);

        f.render_widget(hosts_list, chunks[1]);
    }

    pub fn update(&mut self) {
        if self.ip.is_some() {
            return
        }
        let mut buf = vec![0; 1024];
        let (bytes, recv_addr) = match self.socket.recv_from(&mut buf) {
            Ok(val) => val,
            Err(err) => {
                match err.kind() {
                    ErrorKind::WouldBlock => return,
                    ErrorKind::TimedOut   => return,
                    _ => panic!("{:?}",err)
                }
            }
        };
        let message = match std::str::from_utf8(&buf[0..bytes]) {
            Ok(s) => s,
            Err(_) => return,
        };
        if !message.starts_with(shared::FINDME_PREFIX) { return; }
        let split_msg: Vec<&str> = message.split("=").collect();
        let name = split_msg[1];
        self.found.insert(name.to_string(), recv_addr.ip());

        if name == shared::NAME {
            self.ip = Some(recv_addr.ip());
        }
    }
}

impl Default for FindState {
    fn default() -> Self {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, MULTICAST_PORT)).unwrap();

        let multicast_addr = shared::MULTICAST_ADDR.into();
        let multicast_interface = Ipv4Addr::UNSPECIFIED;
        socket.join_multicast_v4(&multicast_addr, &multicast_interface)
            .expect("Could not join multicast group");

        socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();

        Self { ip: Default::default(), found: Default::default(), socket, multicast_addr}
    }
}