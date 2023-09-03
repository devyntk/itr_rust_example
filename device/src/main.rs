#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod wifi;
mod util;
mod find;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    let mut control = wifi::setup_wifi(p, &spawner).await;

    let delay = Duration::from_secs(1);
    loop {
        log::info!("led on!");
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        log::info!("led off!");
        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}