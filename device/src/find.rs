use shared::{FINDME_STRING, MULTICAST_ADDR, MULTICAST_PORT, NAME, NICE_MUILTICAST};
use embassy_time::{Duration, Timer};
use embassy_net::{Ipv4Address, Stack};
use embassy_net::udp::{PacketMetadata, UdpSocket};

#[embassy_executor::task]
pub async fn broadcast(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut rx_meta_buffer = [PacketMetadata::EMPTY; 16];
    let mut tx_meta_buffer = [PacketMetadata::EMPTY; 16];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta_buffer,
        &mut rx_buffer,
        &mut tx_meta_buffer,
        &mut tx_buffer,
    );
    socket.bind(MULTICAST_PORT).unwrap();
    let addr = (Ipv4Address::from_bytes(&MULTICAST_ADDR), MULTICAST_PORT);
    loop {
        log::info!("sending out findme {:?} on {:?}", NAME, NICE_MUILTICAST);
        socket.send_to(FINDME_STRING.as_bytes(), addr).await.unwrap();
        Timer::after(Duration::from_secs(5)).await;
    }
}