use mlua::{prelude::LuaUserDataMethods, UserData};
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader}, process::{ChildStderr, ChildStdin, ChildStdout}, select};

use super::Status;

pub struct Child {
	inner:  tokio::process::Child,
	stdin:  Option<ChildStdin>,
	stdout: Option<BufReader<ChildStdout>>,
	stderr: Option<BufReader<ChildStderr>>,
}

impl Child {
	pub fn new(mut inner: tokio::process::Child) -> Self {
		let stdin = inner.stdin.take();
		let stdout = inner.stdout.take().map(BufReader::new);
		let stderr = inner.stderr.take().map(BufReader::new);
		Self { inner, stdin, stdout, stderr }
	}
}

impl UserData for Child {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_async_method_mut("read", |_, me, ()| async move {
			async fn read(t: Option<impl AsyncBufReadExt + Unpin>) -> Option<Vec<u8>> {
				let Some(mut r) = t else {
					return None;
				};

				let mut buf = vec![0; 4096];
				match r.read(&mut buf).await {
					Ok(0) | Err(_) => return None,
					Ok(n) => buf.truncate(n),
				}
				Some(buf)
			}

			Ok(select! {
				Some(r) = read(me.stdout.as_mut()) => (0u8, r),
				Some(r) = read(me.stderr.as_mut()) => (1u8, r),
				else => (2u8, Vec::new())
			})
		});
		methods.add_async_method_mut("read_line", |_, me, ()| async move {
			async fn read(t: Option<impl AsyncBufReadExt + Unpin>) -> Option<String> {
				let Some(mut r) = t else {
					return None;
				};

				let mut buf = String::new();
				match r.read_line(&mut buf).await {
					Ok(0) | Err(_) => None,
					Ok(_) => Some(buf),
				}
			}

			Ok(select! {
				Some(r) = read(me.stdout.as_mut()) => (0u8, r),
				Some(r) = read(me.stderr.as_mut()) => (1u8, r),
				else => (2u8, String::new()),
			})
		});
		methods.add_async_method_mut("wait", |_, me, ()| async move {
			Ok(Status::new(me.inner.wait().await?))
		});
		methods.add_method_mut("start_kill", |_, me, ()| Ok(me.inner.start_kill().is_ok()));
	}
}
