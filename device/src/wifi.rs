use shared::{MULTICAST_ADDR, MULTICAST_PORT, FINDME_STRING};

use embassy_time::{Timer, Duration};
use cyw43::Control;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_net::udp::{UdpSocket, PacketMetadata};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_net::{Config, Stack, StackResources, Ipv4Address};
use static_cell::make_static;
use defmt::{unwrap, info};

const WIFI_NETWORK: &str = "SSID";
const WIFI_PASSWORD: &str = "yourpass";

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static, PIN_23>, PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn broadcast(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
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
    // we're not receiving anything on this, so no addr and no port necessary
    unwrap!(socket.bind(0));
    let addr = (Ipv4Address::from_bytes(&MULTICAST_ADDR), MULTICAST_PORT);
    loop{
        unwrap!(socket.send_to(FINDME_STRING.as_bytes(), addr).await);
        Timer::after(Duration::from_secs(1)).await;
    }
}

pub async fn setup_wifi(p: embassy_rp::Peripherals, spawner: &Spawner) -> Control{
    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);

    let state = make_static!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = Config::dhcpv4(Default::default());

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*make_static!(Stack::new(
        net_device,
        config,
        make_static!(StackResources::<2>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    loop {
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }
    control
}