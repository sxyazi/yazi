use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Flags(u32);

bitflags! {
	impl Flags: u32 {
		const READ     = 0b000001;
		const WRITE    = 0b000010;
		const APPEND   = 0b000100;
		const CREATE   = 0b001000;
		const TRUNCATE = 0b010000;
		const EXCLUDE  = 0b100000;
	}
}
