use std::time::Duration;

use mlua::{prelude::LuaUserDataMethods, IntoLua, Table, UserData, Value};
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
		#[inline]
		async fn read_line(me: &mut Child) -> (String, u8) {
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

			select! {
				Some(r) = read(me.stdout.as_mut()) => (r, 0u8),
				Some(r) = read(me.stderr.as_mut()) => (r, 1u8),
				else => (String::new(), 2u8),
			}
		}

		methods.add_async_method_mut("read", |_, me, len: usize| async move {
			async fn read(t: Option<impl AsyncBufReadExt + Unpin>, len: usize) -> Option<Vec<u8>> {
				let Some(mut r) = t else {
					return None;
				};

				let mut buf = vec![0; len];
				match r.read(&mut buf).await {
					Ok(0) | Err(_) => return None,
					Ok(n) => buf.truncate(n),
				}
				Some(buf)
			}

			Ok(select! {
				Some(r) = read(me.stdout.as_mut(), len) => (r, 0u8),
				Some(r) = read(me.stderr.as_mut(), len) => (r, 1u8),
				else => (vec![], 2u8)
			})
		});
		methods.add_async_method_mut("read_line", |_, me, ()| async move { Ok(read_line(me).await) });
		methods.add_async_method_mut("read_line_with", |_, me, options: Table| async move {
			let timeout: u64 = options.get("timeout")?;
			match tokio::time::timeout(Duration::from_millis(timeout), read_line(me)).await {
				Ok(value) => Ok(value),
				Err(_) => Ok((String::new(), 3u8)),
			}
		});
		methods.add_async_method_mut("wait", |lua, me, ()| async move {
			Ok(match me.inner.wait().await {
				Ok(status) => (Status::new(status).into_lua(lua)?, Value::Nil),
				Err(e) => (Value::Nil, e.raw_os_error().into_lua(lua)?),
			})
		});
		methods.add_method_mut("start_kill", |lua, me, ()| {
			Ok(match me.inner.start_kill() {
				Ok(_) => (true, Value::Nil),
				Err(e) => (false, e.raw_os_error().into_lua(lua)?),
			})
		});
	}
}
