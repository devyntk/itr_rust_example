#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod find;
mod util;
mod wifi;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Async};
use cyw43::Control;
use embassy_time::Duration;
use embassy_net::tcp::TcpSocket;
use embassy_net::Stack;
use postcard::experimental::max_size::MaxSize;
use shared::DeviceMsg;
use embedded_io_async::{Write, Read};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let (mut control, stack, mut adc,mut  ts) = wifi::setup_wifi(spawner, p).await;
    loop{
        if let Err(err) = socket_handle(&mut control, stack, &mut adc, &mut ts).await {
            log::error!("DeviceError: {:?}", err);
        }
    }
}

async fn socket_handle(
    control: &mut Control<'static>, 
    stack: &'static Stack<cyw43::NetDriver<'static>>,
    adc: &mut Adc<'static, Async>,
    ts: &mut Channel<'static>
) -> Result<(),DeviceError>{
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut read_buf = [0; shared::ControllerMsg::POSTCARD_MAX_SIZE+1];

    log::info!("Creating TCP socket");
    let mut sock = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    sock.set_timeout(Some(Duration::from_secs(10)));
    loop {
        log::info!("Waiting for TCP connections on port {}", shared::APPLICATION_PORT);
        sock.accept(shared::APPLICATION_PORT).await?;
        log::info!("Got new connection from {:?}", sock.remote_endpoint());
        loop {
            log::info!("Waiting for packet.");
            let read = sock.read(&mut read_buf).await?;
            let controller_msg: shared::ControllerMsg =
                postcard::from_bytes(&mut read_buf[0..read])?;
            log::info!("Received packet: {:?}", controller_msg);
            control.gpio_set(0, controller_msg.light_on).await;

            let temp = adc.read(ts).await?;
            let device_msg: DeviceMsg = DeviceMsg {
                internal_temp: util::convert_to_celsius(temp),
            };
            let slice: heapless::Vec<u8, 32> = postcard::to_vec(&device_msg)?;
            log::info!("Writing packet: {:?} ({:?}, length {:?})", device_msg, slice, slice.len());
            sock.write_all(&slice).await?;
        }
    }
}

#[derive(Debug)]
enum DeviceError{
    Postcard(postcard::Error),
    Tcp(embassy_net::tcp::Error),
    Write(embedded_io_async::WriteAllError<embassy_net::tcp::Error>),
    Read(embedded_io_async::ReadExactError<embassy_net::tcp::Error>),
    Accept(embassy_net::tcp::AcceptError),
    Adc(embassy_rp::adc::Error)
}
impl From<postcard::Error> for DeviceError {
    fn from(value: postcard::Error) -> Self {
        Self::Postcard(value)
    }
}
impl From<embassy_net::tcp::Error> for DeviceError {
    fn from(value: embassy_net::tcp::Error) -> Self {
        Self::Tcp(value)
    }
}
impl From<embassy_net::tcp::AcceptError> for DeviceError {
    fn from(value: embassy_net::tcp::AcceptError) -> Self {
        Self::Accept(value)
    }
}
impl From<embedded_io_async::WriteAllError<embassy_net::tcp::Error>> for DeviceError {
    fn from(value: embedded_io_async::WriteAllError<embassy_net::tcp::Error>) -> Self {
        Self::Write(value)
    }
}impl From<embedded_io_async::ReadExactError<embassy_net::tcp::Error>> for DeviceError {
    fn from(value: embedded_io_async::ReadExactError<embassy_net::tcp::Error>) -> Self {
        Self::Read(value)
    }
}
impl From<embassy_rp::adc::Error> for DeviceError {
    fn from(value: embassy_rp::adc::Error) -> Self {
        Self::Adc(value)
    }
}