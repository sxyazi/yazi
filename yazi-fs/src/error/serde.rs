use std::io;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Error;

fn kind_to_str(kind: io::ErrorKind) -> &'static str {
	use std::io::ErrorKind as K;
	match kind {
		K::NotFound => "not_found",
		K::PermissionDenied => "permission_denied",
		K::ConnectionRefused => "connection_refused",
		K::ConnectionReset => "connection_reset",
		K::HostUnreachable => "host_unreachable",
		K::NetworkUnreachable => "network_unreachable",
		K::ConnectionAborted => "connection_aborted",
		K::NotConnected => "not_connected",
		K::AddrInUse => "addr_in_use",
		K::AddrNotAvailable => "addr_not_available",
		K::NetworkDown => "network_down",
		K::BrokenPipe => "broken_pipe",
		K::AlreadyExists => "already_exists",
		K::WouldBlock => "would_block",
		K::NotADirectory => "not_a_directory",
		K::IsADirectory => "is_a_directory",
		K::DirectoryNotEmpty => "directory_not_empty",
		K::ReadOnlyFilesystem => "read_only_filesystem",
		// K::FilesystemLoop => "filesystem_loop",
		K::StaleNetworkFileHandle => "stale_network_file_handle",
		K::InvalidInput => "invalid_input",
		K::InvalidData => "invalid_data",
		K::TimedOut => "timed_out",
		K::WriteZero => "write_zero",
		K::StorageFull => "storage_full",
		K::NotSeekable => "not_seekable",
		K::QuotaExceeded => "quota_exceeded",
		K::FileTooLarge => "file_too_large",
		K::ResourceBusy => "resource_busy",
		K::ExecutableFileBusy => "executable_file_busy",
		K::Deadlock => "deadlock",
		K::CrossesDevices => "crosses_devices",
		K::TooManyLinks => "too_many_links",
		K::InvalidFilename => "invalid_filename",
		K::ArgumentListTooLong => "argument_list_too_long",
		K::Interrupted => "interrupted",
		K::Unsupported => "unsupported",
		K::UnexpectedEof => "unexpected_eof",
		K::OutOfMemory => "out_of_memory",
		// K::InProgress => "in_progress",
		K::Other => "other",
		_ => "other",
	}
}

fn kind_from_str(s: &str) -> io::ErrorKind {
	use std::io::ErrorKind as K;
	match s {
		"not_found" => K::NotFound,
		"permission_denied" => K::PermissionDenied,
		"connection_refused" => K::ConnectionRefused,
		"connection_reset" => K::ConnectionReset,
		"host_unreachable" => K::HostUnreachable,
		"network_unreachable" => K::NetworkUnreachable,
		"connection_aborted" => K::ConnectionAborted,
		"not_connected" => K::NotConnected,
		"addr_in_use" => K::AddrInUse,
		"addr_not_available" => K::AddrNotAvailable,
		"network_down" => K::NetworkDown,
		"broken_pipe" => K::BrokenPipe,
		"already_exists" => K::AlreadyExists,
		"would_block" => K::WouldBlock,
		"not_a_directory" => K::NotADirectory,
		"is_a_directory" => K::IsADirectory,
		"directory_not_empty" => K::DirectoryNotEmpty,
		"read_only_filesystem" => K::ReadOnlyFilesystem,
		// "filesystem_loop" => K::FilesystemLoop,
		"stale_network_file_handle" => K::StaleNetworkFileHandle,
		"invalid_input" => K::InvalidInput,
		"invalid_data" => K::InvalidData,
		"timed_out" => K::TimedOut,
		"write_zero" => K::WriteZero,
		"storage_full" => K::StorageFull,
		"not_seekable" => K::NotSeekable,
		"quota_exceeded" => K::QuotaExceeded,
		"file_too_large" => K::FileTooLarge,
		"resource_busy" => K::ResourceBusy,
		"executable_file_busy" => K::ExecutableFileBusy,
		"deadlock" => K::Deadlock,
		"crosses_devices" => K::CrossesDevices,
		"too_many_links" => K::TooManyLinks,
		"invalid_filename" => K::InvalidFilename,
		"argument_list_too_long" => K::ArgumentListTooLong,
		"interrupted" => K::Interrupted,
		"unsupported" => K::Unsupported,
		"unexpected_eof" => K::UnexpectedEof,
		"out_of_memory" => K::OutOfMemory,
		// "in_progress" => K::InProgress,
		"other" => K::Other,
		_ => K::Other,
	}
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
			Shadow::Kind { kind } => Self::Kind(kind_from_str(&kind)),
			Shadow::Raw { code } => Self::Raw(code),
			Shadow::Dyn { kind, code, message } => {
				if !message.is_empty() {
					Self::Custom { kind: kind_from_str(&kind), code, message: message.into() }
				} else if let Some(code) = code {
					Self::Raw(code)
				} else {
					Self::Kind(kind_from_str(&kind))
				}
			}
		})
	}
}
