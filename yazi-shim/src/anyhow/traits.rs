use std::io;

pub trait ErrorExt {
	fn is_io_error(&self, kind: io::ErrorKind) -> bool;
}

impl ErrorExt for anyhow::Error {
	fn is_io_error(&self, kind: io::ErrorKind) -> bool {
		self
			.chain()
			.any(|cause| cause.downcast_ref::<io::Error>().is_some_and(|err| err.kind() == kind))
	}
}
