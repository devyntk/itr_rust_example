#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod wifi;
mod util;
mod find;

use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_rp::adc::{Channel, Adc};
use shared::DeviceMsg;
use defmt_rtt as _;
use panic_probe as _;


// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     log::error!("{}", info);
//     loop {}
// }

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let (mut control, stack) = wifi::setup_wifi(&spawner, p).await;

    // let mut adc = Adc::new(p.ADC, crate::wifi::Irqs, embassy_rp::adc::Config::default());
    // let mut ts = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut buf = [0; 1024];

    let mut sock = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    control.gpio_set(0, false).await;
    loop {
        sock.accept(shared::APPLICATION_PORT).await.unwrap();
        loop {
            let bytes = sock.read(&mut buf).await.unwrap();
            let controller_msg: shared::ControllerMsg = postcard::from_bytes(&buf[0..bytes]).unwrap();
            control.gpio_set(0, controller_msg.light_on).await;

            // let temp = adc.read(&mut ts).await.unwrap();
            // let device_msg: DeviceMsg = DeviceMsg { internal_temp: util::convert_to_celsius(temp) };
            // postcard::to_slice(&device_msg, &mut buf).unwrap();
            // sock.write(&buf).await.unwrap();
        }
    }
}
