use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use defmt::{global_logger, unwrap};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::Peripheral;
use embassy_sync::pipe::Pipe;
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::Driver, Builder, UsbDevice};
use static_cell::make_static;

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS: AtomicU8 = AtomicU8::new(0);

pub static mut LOGGER_OBJ: Option<UsbLogger> = None;

const MAX_PACKET_SIZE: u8 = 64;

#[embassy_executor::task]
async fn usb_task(
    mut device: UsbDevice<'static, embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>>
) -> ! {
    device.run().await
}

async fn reader_task() -> ! {
    let mut rx: [u8; MAX_PACKET_SIZE as usize] = [0; MAX_PACKET_SIZE as usize];
    sender.wait_connection().await;
    loop {
        let len = buffer.read(&mut rx[..]).await;
        let _ = sender.write_packet(&rx[..len]).await;
    }
}

pub struct LoggerState<'d> {
    state: embassy_usb::class::cdc_acm::State<'d>,
    device_descriptor: [u8; 32],
    config_descriptor: [u8; 128],
    bos_descriptor: [u8; 16],
    control_buf: [u8; 64],
}

impl<'d> LoggerState<'d> {
    /// Create a new instance of the logger state.
    pub fn new() -> Self {
        Self {
            state: embassy_usb::class::cdc_acm::State::new(),
            device_descriptor: [0; 32],
            config_descriptor: [0; 128],
            bos_descriptor: [0; 16],
            control_buf: [0; 64],
        }
    }
}

pub struct UsbLogger<> {
    buffer: Pipe<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, 4096>,
}

impl UsbLogger {
    /// Create a new logger instance.
    pub const fn new() -> Self {
        Self { 
            buffer: Pipe::new(),
        }
    }

    pub fn write<'a>(&mut self, buf: &'a [u8]) {
        self.buffer.try_write(buf).unwrap();
    }

    pub fn flush(&self) {

    }

    /// Run the USB logger using the state and USB driver. Never returns.
    pub async fn run<'d, D>(
        &'d self, state: 
        &'d mut LoggerState<'d>, 
        driver: D,
        spawner: &Spawner,
    ) 
    where
        D: Driver<'d>,
        Self: 'd,
    {

        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("ITR");
        config.product = Some("USB-serial logger");
        config.serial_number = None;
        config.max_power = 100;
        config.max_packet_size_0 = MAX_PACKET_SIZE;

        // Required for windows compatiblity.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        let mut builder = Builder::new(
            driver,
            config,
            &mut state.device_descriptor,
            &mut state.config_descriptor,
            &mut state.bos_descriptor,
            &mut state.control_buf,
        );

        // Create classes on the builder.
        let class = CdcAcmClass::new(&mut builder, &mut state.state, MAX_PACKET_SIZE as u16);
        let (mut sender, mut receiver) = class.split();

        // Build the builder.
        let mut device = builder.build();

        spawner.spawn(usb_task(device));
        loop {
            let log_fut = async {
                let mut rx: [u8; MAX_PACKET_SIZE as usize] = [0; MAX_PACKET_SIZE as usize];
                sender.wait_connection().await;
                loop {
                    let len = self.buffer.read(&mut rx[..]).await;
                    let _ = sender.write_packet(&rx[..len]).await;
                }
            };
            let discard_fut = async {
                let mut discard_buf: [u8; MAX_PACKET_SIZE as usize] = [0; MAX_PACKET_SIZE as usize];
                receiver.wait_connection().await;
                loop {
                    let _ = receiver.read_packet(&mut discard_buf).await;
                }
            };
            join(log_fut, discard_fut).await;
        }
    }
}

#[global_logger]
struct GlobalSerialLogger;

unsafe impl defmt::Logger for GlobalSerialLogger {
    fn acquire() {
        let token = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly");
        }

        TAKEN.store(true, Ordering::Relaxed);

        INTERRUPTS.store(token, Ordering::Relaxed);
        unsafe{
            if let Some(ref mut logger) = &mut LOGGER_OBJ {
                ENCODER.start_frame(|data|logger.write(data))
            }
        }
    }

    unsafe fn release() {
        if let Some(ref mut logger) = &mut LOGGER_OBJ {
            ENCODER.end_frame(|data|logger.write(data))
        }
        TAKEN.store(false, Ordering::Relaxed);
        critical_section::release(INTERRUPTS.load(Ordering::Relaxed));
    }

    unsafe fn write(bytes: &[u8]) {
        if let Some(ref mut logger) = &mut LOGGER_OBJ {
            ENCODER.write(bytes,|data|logger.write(data))
        }
    }

    unsafe fn flush() {
        if let Some(ref mut logger) = &mut LOGGER_OBJ {
            logger.flush()
        }
    }
}

pub async fn setup_logging(spawner: &Spawner, p: embassy_rp::Peripherals) -> embassy_rp::Peripherals{
    unsafe {
        let driver = embassy_rp::usb::Driver::new(p.USB.clone_unchecked(), crate::wifi::Irqs);
        let token = critical_section::acquire();
        LOGGER_OBJ = Some(crate::logging::UsbLogger::new());
        critical_section::release(token);
        if let Some(ref mut logger) = &mut LOGGER_OBJ {
            logger.run(&mut crate::logging::LoggerState::new(), driver, spawner);
        }
    }
    p
}