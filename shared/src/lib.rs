#![no_std]
use const_format::{concatcp, assertcp_ne};

// +================+
//     CHANGE ME
// +================+
const NAME: &str = "changeme";
// +================+
//   GO CHANGE THAT
// +================+

assertcp_ne!(NAME, "changeme", "you must change the NAME parameter");
pub const FINDME_STRING: &str = concatcp!("findme:name=", NAME);
pub const MULTICAST_ADDR: [u8; 4] = [239,255,70,77];
pub const MULTICAST_PORT: u16 = 50765;

pub const APPLICATION_PORT: u16 = 50767;
