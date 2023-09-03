#![no_std]
use const_format::{assertcp_ne, concatcp};

// +================+
//     CHANGE ME
// +================+
pub const NAME: &str = "devyn";
// +================+
//   GO CHANGE THAT
// +================+

assertcp_ne!(NAME, "changeme", "you must change the NAME parameter");
pub const FINDME_PREFIX: &str = "findme:name=";
pub const FINDME_STRING: &str = concatcp!(FINDME_PREFIX, NAME);
pub const MULTICAST_ADDR: [u8; 4] = [239, 255, 70, 77];
pub const MULTICAST_PORT: u16 = 50765;

pub const APPLICATION_PORT: u16 = 50767;
