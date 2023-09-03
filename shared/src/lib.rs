#![no_std]
use const_format::{assertcp_ne, concatcp};
use serde::{Serialize, Deserialize};

// +================+
//     CHANGE ME
// +================+
pub const NAME: &str = "devyn";
// +================+
//   GO CHANGE THAT
// +================+

#[derive(Serialize, Deserialize)]
pub struct ControllerMsg{
    pub light_on: bool
}

#[derive(Serialize, Deserialize)]
pub struct DeviceMsg{
    pub internal_temp: f32
}

assertcp_ne!(NAME, "changeme", "you must change the NAME parameter");
pub const FINDME_PREFIX: &str = "findme:name=";
pub const FINDME_STRING: &str = concatcp!(FINDME_PREFIX, NAME);
pub const MULTICAST_ADDR: [u8; 4] = [239, 255, 70, 77];
pub const MULTICAST_PORT: u16 = 50765;

pub const APPLICATION_PORT: u16 = 50767;
