#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod find;
mod util;
mod wifi;

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let (mut control, stack, mut adc, mut ts) = wifi::setup_wifi(spawner, p).await;
}
