#[allow(dead_code)]
pub fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}

#[derive(Debug)]
enum DeviceError {
    Postcard(postcard::Error),
    Tcp(embassy_net::tcp::Error),
    Write(embedded_io_async::WriteAllError<embassy_net::tcp::Error>),
    Read(embedded_io_async::ReadExactError<embassy_net::tcp::Error>),
    Accept(embassy_net::tcp::AcceptError),
    Adc(embassy_rp::adc::Error),
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
}
impl From<embedded_io_async::ReadExactError<embassy_net::tcp::Error>> for DeviceError {
    fn from(value: embedded_io_async::ReadExactError<embassy_net::tcp::Error>) -> Self {
        Self::Read(value)
    }
}
impl From<embassy_rp::adc::Error> for DeviceError {
    fn from(value: embassy_rp::adc::Error) -> Self {
        Self::Adc(value)
    }
}
