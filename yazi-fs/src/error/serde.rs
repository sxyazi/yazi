use std::io;

use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Error;

pub(super) fn kind_to_str(kind: io::ErrorKind) -> &'static str {
	use std::io::ErrorKind as K;
	match kind {
		K::NotFound => "NotFound",
		K::PermissionDenied => "PermissionDenied",
		K::ConnectionRefused => "ConnectionRefused",
		K::ConnectionReset => "ConnectionReset",
		K::HostUnreachable => "HostUnreachable",
		K::NetworkUnreachable => "NetworkUnreachable",
		K::ConnectionAborted => "ConnectionAborted",
		K::NotConnected => "NotConnected",
		K::AddrInUse => "AddrInUse",
		K::AddrNotAvailable => "AddrNotAvailable",
		K::NetworkDown => "NetworkDown",
		K::BrokenPipe => "BrokenPipe",
		K::AlreadyExists => "AlreadyExists",
		K::WouldBlock => "WouldBlock",
		K::NotADirectory => "NotADirectory",
		K::IsADirectory => "IsADirectory",
		K::DirectoryNotEmpty => "DirectoryNotEmpty",
		K::ReadOnlyFilesystem => "ReadOnlyFilesystem",
		// K::FilesystemLoop => "FilesystemLoop",
		K::StaleNetworkFileHandle => "StaleNetworkFileHandle",
		K::InvalidInput => "InvalidInput",
		K::InvalidData => "InvalidData",
		K::TimedOut => "TimedOut",
		K::WriteZero => "WriteZero",
		K::StorageFull => "StorageFull",
		K::NotSeekable => "NotSeekable",
		K::QuotaExceeded => "QuotaExceeded",
		K::FileTooLarge => "FileTooLarge",
		K::ResourceBusy => "ResourceBusy",
		K::ExecutableFileBusy => "ExecutableFileBusy",
		K::Deadlock => "Deadlock",
		K::CrossesDevices => "CrossesDevices",
		K::TooManyLinks => "TooManyLinks",
		K::InvalidFilename => "InvalidFilename",
		K::ArgumentListTooLong => "ArgumentListTooLong",
		K::Interrupted => "Interrupted",
		K::Unsupported => "Unsupported",
		K::UnexpectedEof => "UnexpectedEof",
		K::OutOfMemory => "OutOfMemory",
		// K::InProgress => "InProgress",
		K::Other => "Other",
		_ => "Other",
	}
}

pub(super) fn kind_from_str(s: &str) -> Result<io::ErrorKind> {
	use std::io::ErrorKind as K;
	Ok(match s {
		"NotFound" => K::NotFound,
		"PermissionDenied" => K::PermissionDenied,
		"ConnectionRefused" => K::ConnectionRefused,
		"ConnectionReset" => K::ConnectionReset,
		"HostUnreachable" => K::HostUnreachable,
		"NetworkUnreachable" => K::NetworkUnreachable,
		"ConnectionAborted" => K::ConnectionAborted,
		"NotConnected" => K::NotConnected,
		"AddrInUse" => K::AddrInUse,
		"AddrNotAvailable" => K::AddrNotAvailable,
		"NetworkDown" => K::NetworkDown,
		"BrokenPipe" => K::BrokenPipe,
		"AlreadyExists" => K::AlreadyExists,
		"WouldBlock" => K::WouldBlock,
		"NotADirectory" => K::NotADirectory,
		"IsADirectory" => K::IsADirectory,
		"DirectoryNotEmpty" => K::DirectoryNotEmpty,
		"ReadOnlyFilesystem" => K::ReadOnlyFilesystem,
		// "FilesystemLoop" => K::FilesystemLoop,
		"StaleNetworkFileHandle" => K::StaleNetworkFileHandle,
		"InvalidInput" => K::InvalidInput,
		"InvalidData" => K::InvalidData,
		"TimedOut" => K::TimedOut,
		"WriteZero" => K::WriteZero,
		"StorageFull" => K::StorageFull,
		"NotSeekable" => K::NotSeekable,
		"QuotaExceeded" => K::QuotaExceeded,
		"FileTooLarge" => K::FileTooLarge,
		"ResourceBusy" => K::ResourceBusy,
		"ExecutableFileBusy" => K::ExecutableFileBusy,
		"Deadlock" => K::Deadlock,
		"CrossesDevices" => K::CrossesDevices,
		"TooManyLinks" => K::TooManyLinks,
		"InvalidFilename" => K::InvalidFilename,
		"ArgumentListTooLong" => K::ArgumentListTooLong,
		"Interrupted" => K::Interrupted,
		"Unsupported" => K::Unsupported,
		"UnexpectedEof" => K::UnexpectedEof,
		"OutOfMemory" => K::OutOfMemory,
		// "InProgress" => K::InProgress,
		"Other" => K::Other,
		_ => bail!("unknown error kind: {s}"),
	})
}

impl Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		#[derive(Serialize)]
		#[serde(tag = "type", rename_all = "kebab-case")]
		enum Shadow<'a> {
			Kind { kind: &'a str },
			Raw { code: i32 },
			Dyn { kind: &'a str, code: Option<i32>, message: &'a str },
		}

		match self {
			Self::Kind(kind) => Shadow::Kind { kind: kind_to_str(*kind) }.serialize(serializer),
			Self::Raw(code) => Shadow::Raw { code: *code }.serialize(serializer),
			Self::Custom { kind, code, message } => {
				Shadow::Dyn { kind: kind_to_str(*kind), code: *code, message }.serialize(serializer)
			}
		}
	}
}

impl<'de> Deserialize<'de> for Error {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		#[serde(tag = "type", rename_all = "kebab-case")]
		enum Shadow {
			Kind { kind: String },
			Raw { code: i32 },
			Dyn { kind: String, code: Option<i32>, message: String },
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(match shadow {
			Shadow::Kind { kind } => Self::Kind(kind_from_str(&kind).map_err(serde::de::Error::custom)?),
			Shadow::Raw { code } => Self::Raw(code),
			Shadow::Dyn { kind, code, message } => {
				if !message.is_empty() {
					Self::Custom {
						kind: kind_from_str(&kind).map_err(serde::de::Error::custom)?,
						code,
						message: message.into(),
					}
				} else if let Some(code) = code {
					Self::Raw(code)
				} else {
					Self::Kind(kind_from_str(&kind).map_err(serde::de::Error::custom)?)
				}
			}
		})
	}
}
