/// The id of a peer in the DDS system.
#[derive(Debug, PartialEq)]
pub enum DDSPeer {
	/// Internally, `0` is used to represent all peers.
	All,
	One(u64),
}

impl DDSPeer {
	pub fn matches(&self, peer_id: u64) -> bool {
		match self {
			Self::All => true,
			Self::One(id) => *id == peer_id,
		}
	}
}

impl From<u64> for DDSPeer {
	fn from(value: u64) -> Self {
		match value {
			0 => Self::All,
			_ => Self::One(value),
		}
	}
}
