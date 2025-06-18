use std::{any::TypeId, io, process::Stdio};

use mlua::{AnyUserData, ExternalError, IntoLuaMulti, Lua, MetaMethod, Table, UserData, Value};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use yazi_binding::Error;
use yazi_shared::IntoOsStr;

use super::{Child, output::Output};
use crate::process::Status;

pub struct Command {
	inner:  tokio::process::Command,
	memory: Option<usize>,
}

const NULL: u8 = 0;
const PIPED: u8 = 1;
const INHERIT: u8 = 2;

impl Command {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, program): (Table, String)| {
			let mut inner = tokio::process::Command::new(program);
			inner.kill_on_drop(true).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

			Ok(Self { inner, memory: None })
		})?;

		let command = lua.create_table_from([
			// Stdio
			("NULL", NULL),
			("PIPED", PIPED),
			("INHERIT", INHERIT),
		])?;

		command.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		lua.globals().raw_set("Command", command)
	}

	#[cfg(unix)]
	fn spawn(&mut self) -> io::Result<Child> {
		if let Some(max) = self.memory {
			unsafe {
				self.inner.pre_exec(move || {
					let rlp = libc::rlimit { rlim_cur: max as _, rlim_max: max as _ };
					libc::setrlimit(libc::RLIMIT_AS, &rlp);
					Ok(())
				});
			}
		}
		self.inner.spawn().map(Child::new)
	}

	#[cfg(windows)]
	fn spawn(&mut self) -> io::Result<Child> {
		use std::os::windows::io::RawHandle;

		use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject};

		fn assign_job(max: usize, handle: RawHandle) -> io::Result<RawHandle> {
			unsafe {
				let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
				if job.is_null() {
					return Err(io::Error::last_os_error());
				}

				let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
				info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY;
				info.ProcessMemoryLimit = max;

				let result = SetInformationJobObject(
					job,
					JobObjectExtendedLimitInformation,
					&mut info as *mut _ as *mut _,
					std::mem::size_of_val(&info) as u32,
				);

				if result == 0 {
					Err(io::Error::last_os_error())
				} else if AssignProcessToJobObject(job, handle) == 0 {
					Err(io::Error::last_os_error())
				} else {
					Ok(job)
				}
			}
		}

		let child = self.inner.spawn()?;
		if let (Some(max), Some(handle)) = (self.memory, child.raw_handle()) {
			if let Ok(job) = assign_job(max, handle) {
				return Ok(Child::new(child, Some(job)));
			}
		}

		Ok(Child::new(child, None))
	}

	async fn output(&mut self) -> io::Result<std::process::Output> {
		self.inner.stdin(Stdio::piped());
		self.inner.stdout(Stdio::piped());
		self.spawn()?.wait_with_output().await
	}

	async fn status(&mut self) -> io::Result<std::process::ExitStatus> {
		self.spawn()?.status().await
	}
}

impl UserData for Command {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		#[inline]
		fn make_stdio(v: Value) -> mlua::Result<Stdio> {
			match v {
				Value::Integer(n) => {
					return Ok(match n as u8 {
						PIPED => Stdio::piped(),
						INHERIT => Stdio::inherit(),
						_ => Stdio::null(),
					});
				}
				Value::UserData(ud) => match ud.type_id() {
					Some(t) if t == TypeId::of::<ChildStdin>() => {
						return Ok(ud.take::<ChildStdin>()?.try_into()?);
					}
					Some(t) if t == TypeId::of::<ChildStdout>() => {
						return Ok(ud.take::<ChildStdout>()?.try_into()?);
					}
					Some(t) if t == TypeId::of::<ChildStderr>() => {
						return Ok(ud.take::<ChildStderr>()?.try_into()?);
					}
					_ => {}
				},
				_ => {}
			}

			Err(
				"must be one of Command.NULL, Command.PIPED, Command.INHERIT, or a ChildStdin, ChildStdout, or ChildStderr".into_lua_err(),
			)
		}

		methods.add_function_mut("arg", |_, (ud, arg): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				match arg {
					Value::String(s) => {
						me.inner.arg(s.as_bytes().into_os_str()?);
					}
					Value::Table(t) => {
						for s in t.sequence_values::<mlua::String>() {
							me.inner.arg(s?.as_bytes().into_os_str()?);
						}
					}
					_ => return Err("arg must be a string or table of strings".into_lua_err()),
				}
			}
			Ok(ud)
		});
		methods.add_function_mut("cwd", |_, (ud, dir): (AnyUserData, mlua::String)| {
			ud.borrow_mut::<Self>()?.inner.current_dir(dir.to_str()?.as_ref());
			Ok(ud)
		});
		methods.add_function_mut(
			"env",
			|_, (ud, key, value): (AnyUserData, mlua::String, mlua::String)| {
				ud.borrow_mut::<Self>()?
					.inner
					.env(key.as_bytes().into_os_str()?, value.as_bytes().into_os_str()?);
				Ok(ud)
			},
		);
		methods.add_function_mut("stdin", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stdin(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_function_mut("stdout", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stdout(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_function_mut("stderr", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stderr(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_function_mut("memory", |_, (ud, max): (AnyUserData, usize)| {
			ud.borrow_mut::<Self>()?.memory = Some(max);
			Ok(ud)
		});
		methods.add_method_mut("spawn", |lua, me, ()| match me.spawn() {
			Ok(child) => child.into_lua_multi(lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(lua),
		});
		methods.add_async_method_mut("output", |lua, mut me, ()| async move {
			match me.output().await {
				Ok(output) => Output::new(output).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("status", |lua, mut me, ()| async move {
			match me.status().await {
				Ok(status) => Status::new(status).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
