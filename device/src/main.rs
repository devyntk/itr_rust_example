#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod wifi;
mod util;
mod find;
mod logging;

use cyw43::Control;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use thiserror::Error;
use {panic_probe as _};


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let p = logging::setup_logging(&spawner, p).await;
    let (mut control, stack) = wifi::setup_wifi(p, &spawner).await;

    control.gpio_set(0, true).await;

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut sock = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    loop {
        handle_socket(&mut sock, &mut control).await;
    }
}

async fn handle_socket<'a>(sock: &mut TcpSocket<'a>, control: &mut Control<'a>) -> Result<(), DeviceError> {
    let mut buf = [0; 1024];
    sock.accept(shared::APPLICATION_PORT).await.unwrap();
    loop {
        let bytes = sock.read(&mut buf).await?;
        let controller_msg: shared::ControllerMsg = postcard::from_bytes(&buf[0..bytes])?;
        control.gpio_set(0, controller_msg.light_on).await
    }
}


#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("TCP Error: {0:?}")]
    Disconnect(embassy_net::tcp::Error),
    #[error("Serialization error: {0:?}")]
    Serialization(postcard::Error),
}

impl From<embassy_net::tcp::Error> for DeviceError {
    fn from(value: embassy_net::tcp::Error) -> Self {
        DeviceError::Disconnect(value)
    }
}
impl From<postcard::Error> for DeviceError {
    fn from(value: postcard::Error) -> Self {
        DeviceError::Serialization(value)
    }
}