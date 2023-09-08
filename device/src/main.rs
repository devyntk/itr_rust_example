#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod find;
mod util;
mod wifi;

use cyw43::Control;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let (mut control, stack, _adc, _ts) = wifi::setup_wifi(spawner, p).await;

    loop {
        if let Err(err) = handle_socket(&mut control, stack).await {
            log::error!("Handle func error: {:?}", err)
        }
    }
}

async fn handle_socket(
    control: &mut Control<'static>,
    stack: &'static Stack<cyw43::NetDriver<'static>>,
) -> Result<(), util::DeviceError> {
    loop {
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_secs(1)).await;
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
