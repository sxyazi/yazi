use serde::{Deserialize, Serialize};

use super::de::Deserializer;
use crate::{Error, impl_from_packet, impl_try_from_packet, requests, responses};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Packet<'a> {
	Init(requests::Init),
	Open(requests::Open<'a>),
	Close(requests::Close<'a>),
	Read(requests::Read<'a>),
	Write(requests::Write<'a>),
	Lstat(requests::Lstat<'a>),
	Fstat(requests::Fstat<'a>),
	SetStat(requests::SetStat<'a>),
	FSetStat(requests::FSetStat<'a>),
	OpenDir(requests::OpenDir<'a>),
	ReadDir(requests::ReadDir<'a>),
	Remove(requests::Remove<'a>),
	Mkdir(requests::Mkdir<'a>),
	Rmdir(requests::Rmdir<'a>),
	Realpath(requests::Realpath<'a>),
	Stat(requests::Stat<'a>),
	Rename(requests::Rename<'a>),
	Readlink(requests::Readlink<'a>),
	Symlink(requests::Symlink<'a>),
	ExtendedRename(requests::Extended<'a, requests::ExtendedRename<'a>>),
	ExtendedFsync(requests::Extended<'a, requests::ExtendedFsync<'a>>),
	ExtendedHardlink(requests::Extended<'a, requests::ExtendedHardlink<'a>>),
	ExtendedLimits(requests::Extended<'a, requests::ExtendedLimits>),

	// Responses
	Version(responses::Version),
	Status(responses::Status),
	Handle(responses::Handle),
	Data(responses::Data),
	Name(responses::Name<'a>),
	Attrs(responses::Attrs),
	ExtendedReply(responses::Extended<'a>),
}

impl_from_packet! {
	Init(requests::Init),
	Open(requests::Open<'a>),
	Close(requests::Close<'a>),
	Read(requests::Read<'a>),
	Write(requests::Write<'a>),
	Lstat(requests::Lstat<'a>),
	Fstat(requests::Fstat<'a>),
	SetStat(requests::SetStat<'a>),
	FSetStat(requests::FSetStat<'a>),
	OpenDir(requests::OpenDir<'a>),
	ReadDir(requests::ReadDir<'a>),
	Remove(requests::Remove<'a>),
	Mkdir(requests::Mkdir<'a>),
	Rmdir(requests::Rmdir<'a>),
	Realpath(requests::Realpath<'a>),
	Stat(requests::Stat<'a>),
	Rename(requests::Rename<'a>),
	Readlink(requests::Readlink<'a>),
	Symlink(requests::Symlink<'a>),
	ExtendedRename(requests::Extended<'a, requests::ExtendedRename<'a>>),
	ExtendedFsync(requests::Extended<'a, requests::ExtendedFsync<'a>>),
	ExtendedHardlink(requests::Extended<'a, requests::ExtendedHardlink<'a>>),
	ExtendedLimits(requests::Extended<'a, requests::ExtendedLimits>),

	// Responses
	Version(responses::Version),
	Status(responses::Status),
	Handle(responses::Handle),
	Data(responses::Data),
	Name(responses::Name<'a>),
	Attrs(responses::Attrs),
	ExtendedReply(responses::Extended<'a>),
}

impl_try_from_packet! {
	Version(responses::Version),
	Status(responses::Status),
	Handle(responses::Handle),
	Data(responses::Data),
	Name(responses::Name<'a>),
	Attrs(responses::Attrs),
	ExtendedReply(responses::Extended<'a>),
}

impl Packet<'_> {
	fn kind(&self) -> u8 {
		match self {
			Self::Init(_) => 1,
			Self::Open(_) => 3,
			Self::Close(_) => 4,
			Self::Read(_) => 5,
			Self::Write(_) => 6,
			Self::Lstat(_) => 7,
			Self::Fstat(_) => 8,
			Self::SetStat(_) => 9,
			Self::FSetStat(_) => 10,
			Self::OpenDir(_) => 11,
			Self::ReadDir(_) => 12,
			Self::Remove(_) => 13,
			Self::Mkdir(_) => 14,
			Self::Rmdir(_) => 15,
			Self::Realpath(_) => 16,
			Self::Stat(_) => 17,
			Self::Rename(_) => 18,
			Self::Readlink(_) => 19,
			Self::Symlink(_) => 20,
			Self::ExtendedRename(_) => 200,
			Self::ExtendedFsync(_) => 200,
			Self::ExtendedHardlink(_) => 200,
			Self::ExtendedLimits(_) => 200,

			// Responses
			Self::Version(_) => 2,
			Self::Status(_) => 101,
			Self::Handle(_) => 102,
			Self::Data(_) => 103,
			Self::Name(_) => 104,
			Self::Attrs(_) => 105,
			Self::ExtendedReply(_) => 201,
		}
	}

	pub fn id(&self) -> u32 {
		match self {
			Self::Init(_) => 0,
			Self::Open(v) => v.id,
			Self::Close(v) => v.id,
			Self::Read(v) => v.id,
			Self::Write(v) => v.id,
			Self::Lstat(v) => v.id,
			Self::Fstat(v) => v.id,
			Self::SetStat(v) => v.id,
			Self::FSetStat(v) => v.id,
			Self::OpenDir(v) => v.id,
			Self::ReadDir(v) => v.id,
			Self::Remove(v) => v.id,
			Self::Mkdir(v) => v.id,
			Self::Rmdir(v) => v.id,
			Self::Realpath(v) => v.id,
			Self::Stat(v) => v.id,
			Self::Rename(v) => v.id,
			Self::Readlink(v) => v.id,
			Self::Symlink(v) => v.id,
			Self::ExtendedRename(v) => v.id,
			Self::ExtendedFsync(v) => v.id,
			Self::ExtendedHardlink(v) => v.id,
			Self::ExtendedLimits(v) => v.id,

			// Responses
			Self::Version(_) => 0,
			Self::Status(v) => v.id,
			Self::Handle(v) => v.id,
			Self::Data(v) => v.id,
			Self::Name(v) => v.id,
			Self::Attrs(v) => v.id,
			Self::ExtendedReply(v) => v.id,
		}
	}

	pub fn with_id(mut self, id: u32) -> Self {
		match &mut self {
			Self::Init(_) => {}
			Self::Open(v) => v.id = id,
			Self::Close(v) => v.id = id,
			Self::Read(v) => v.id = id,
			Self::Write(v) => v.id = id,
			Self::Lstat(v) => v.id = id,
			Self::Fstat(v) => v.id = id,
			Self::SetStat(v) => v.id = id,
			Self::FSetStat(v) => v.id = id,
			Self::OpenDir(v) => v.id = id,
			Self::ReadDir(v) => v.id = id,
			Self::Remove(v) => v.id = id,
			Self::Mkdir(v) => v.id = id,
			Self::Rmdir(v) => v.id = id,
			Self::Realpath(v) => v.id = id,
			Self::Stat(v) => v.id = id,
			Self::Rename(v) => v.id = id,
			Self::Readlink(v) => v.id = id,
			Self::Symlink(v) => v.id = id,
			Self::ExtendedRename(v) => v.id = id,
			Self::ExtendedFsync(v) => v.id = id,
			Self::ExtendedHardlink(v) => v.id = id,
			Self::ExtendedLimits(v) => v.id = id,

			// Responses
			Self::Version(_) => {}
			Self::Status(v) => v.id = id,
			Self::Handle(v) => v.id = id,
			Self::Data(v) => v.id = id,
			Self::Name(v) => v.id = id,
			Self::Attrs(v) => v.id = id,
			Self::ExtendedReply(v) => v.id = id,
		}
		self
	}

	fn len(&self) -> usize {
		let type_len = 1;
		match self {
			Self::Init(v) => type_len + v.len(),
			Self::Open(v) => type_len + v.len(),
			Self::Close(v) => type_len + v.len(),
			Self::Read(v) => type_len + v.len(),
			Self::Write(v) => type_len + v.len(),
			Self::Lstat(v) => type_len + v.len(),
			Self::Fstat(v) => type_len + v.len(),
			Self::SetStat(v) => type_len + v.len(),
			Self::FSetStat(v) => type_len + v.len(),
			Self::OpenDir(v) => type_len + v.len(),
			Self::ReadDir(v) => type_len + v.len(),
			Self::Remove(v) => type_len + v.len(),
			Self::Mkdir(v) => type_len + v.len(),
			Self::Rmdir(v) => type_len + v.len(),
			Self::Realpath(v) => type_len + v.len(),
			Self::Stat(v) => type_len + v.len(),
			Self::Rename(v) => type_len + v.len(),
			Self::Readlink(v) => type_len + v.len(),
			Self::Symlink(v) => type_len + v.len(),
			Self::ExtendedRename(v) => type_len + v.len(),
			Self::ExtendedFsync(v) => type_len + v.len(),
			Self::ExtendedHardlink(v) => type_len + v.len(),
			Self::ExtendedLimits(v) => type_len + v.len(),

			// Responses
			Self::Version(v) => type_len + v.len(),
			Self::Status(v) => type_len + v.len(),
			Self::Handle(v) => type_len + v.len(),
			Self::Data(v) => type_len + v.len(),
			Self::Name(v) => type_len + v.len(),
			Self::Attrs(v) => type_len + v.len(),
			Self::ExtendedReply(v) => type_len + v.len(),
		}
	}
}

pub fn to_bytes<'a, T>(value: T) -> Result<Vec<u8>, Error>
where
	T: Into<Packet<'a>> + Serialize,
{
	let packet: Packet = value.into();

	let len = u32::try_from(packet.len()).map_err(|_| Error::serde("packet too large"))?;

	let mut output = Vec::with_capacity(4 + len as usize);
	output.extend_from_slice(&len.to_be_bytes());
	output.push(packet.kind());

	let mut serializer = crate::Serializer { output };
	packet.serialize(&mut serializer)?;
	Ok(serializer.output)
}

// TODO: use Vec<u8>
pub fn from_bytes(mut bytes: &[u8]) -> Result<Packet<'static>, Error> {
	let kind = *bytes.first().ok_or(Error::serde("empty packet"))?;
	bytes = &bytes[1..];

	Ok(match kind {
		2 => Packet::Version(Deserializer::once(bytes)?),
		101 => Packet::Status(Deserializer::once(bytes)?),
		102 => Packet::Handle(Deserializer::once(bytes)?),
		103 => Packet::Data(Deserializer::once(bytes)?),
		104 => Packet::Name(Deserializer::once(bytes)?),
		105 => Packet::Attrs(Deserializer::once(bytes)?),
		201 => Packet::ExtendedReply(Deserializer::once(bytes)?),
		_ => return Err(Error::Packet("unknown packet kind")),
	})
}
