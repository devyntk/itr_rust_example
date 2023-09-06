#![no_std]
use const_format::{assertcp_ne, concatcp};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

// +================+
//     CHANGE ME
// +================+
pub const NAME: &str = "changeme";
// +================+
//   GO CHANGE THAT
// +================+

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub struct ControllerMsg {}

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub struct DeviceMsg {}

assertcp_ne!(NAME, "changeme", "you must change the NAME parameter");
pub const FINDME_PREFIX: &str = "findme:name=";
pub const FINDME_STRING: &str = concatcp!(FINDME_PREFIX, NAME);
pub const MULTICAST_ADDR: [u8; 4] = [239, 255, 70, 77];
pub const MULTICAST_PORT: u16 = 50765;
pub const NICE_MUILTICAST: &str = concatcp!(
    MULTICAST_ADDR[0],
    ".",
    MULTICAST_ADDR[1],
    ".",
    MULTICAST_ADDR[2],
    ".",
    MULTICAST_ADDR[3],
    ":",
    MULTICAST_PORT
);

pub const APPLICATION_PORT: u16 = 50767;
