use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mlua::{AnyUserData, ExternalError, Lua, Table, UserDataFields, UserDataMethods};
use yazi_shared::fs::ChaKind;

use crate::{RtRef, bindings::Cast};

pub struct Cha;

static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[inline]
fn warn_deprecated(id: Option<&str>) {
	if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		let id = match id {
			Some(id) => format!("`{id}.yazi` plugin"),
			None => "`init.lua` config".to_owned(),
		};
		let s = "The `created`, `modified`, `accessed`, `length`, and `permissions` properties of `Cha` have been renamed in Yazi v0.4.

Please use the new `btime`, `mtime`, `atime`, `len`, and `perm` instead, in your {id}. See https://github.com/sxyazi/yazi/pull/1761 for details.";
		yazi_proxy::AppProxy::notify(yazi_proxy::options::NotifyOpt {
			title:   "Deprecated API".to_owned(),
			content: s.replace("{id}", &id),
			level:   yazi_proxy::options::NotifyLevel::Warn,
			timeout: Duration::from_secs(20),
		});
	}
}

impl Cha {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::Cha>(|reg| {
			reg.add_field_method_get("is_dir", |_, me| Ok(me.is_dir()));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
			reg.add_field_method_get("is_orphan", |_, me| Ok(me.is_orphan()));
			reg.add_field_method_get("is_dummy", |_, me| Ok(me.is_dummy()));
			reg.add_field_method_get("is_block", |_, me| Ok(me.is_block()));
			reg.add_field_method_get("is_char", |_, me| Ok(me.is_char()));
			reg.add_field_method_get("is_fifo", |_, me| Ok(me.is_fifo()));
			reg.add_field_method_get("is_sock", |_, me| Ok(me.is_sock()));
			reg.add_field_method_get("is_exec", |_, me| Ok(me.is_exec()));
			reg.add_field_method_get("is_sticky", |_, me| Ok(me.is_sticky()));

			#[cfg(unix)]
			{
				reg.add_field_method_get("uid", |_, me| Ok((!me.is_dummy()).then_some(me.uid)));
				reg.add_field_method_get("gid", |_, me| Ok((!me.is_dummy()).then_some(me.gid)));
				reg.add_field_method_get("nlink", |_, me| Ok((!me.is_dummy()).then_some(me.nlink)));
			}

			reg.add_field_method_get("len", |_, me| Ok(me.len));
			reg.add_field_method_get("atime", |_, me| {
				Ok(me.atime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_field_method_get("btime", |_, me| {
				Ok(me.btime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			#[cfg(unix)]
			reg.add_field_method_get("ctime", |_, me| {
				Ok(me.ctime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_field_method_get("mtime", |_, me| {
				Ok(me.mtime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_method("perm", |_, _me, ()| {
				Ok(
					#[cfg(unix)]
					Some(yazi_shared::fs::permissions(_me.mode, _me.is_dummy())),
					#[cfg(windows)]
					None::<String>,
				)
			});

			// TODO: remove these deprecated properties in the future
			{
				reg.add_field_method_get("length", |lua, me| {
					warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
					Ok(me.len)
				});
				reg.add_field_method_get("created", |lua, me| {
					warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
					Ok(me.btime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
				});
				reg.add_field_method_get("modified", |lua, me| {
					warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
					Ok(me.mtime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
				});
				reg.add_field_method_get("accessed", |lua, me| {
					warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
					Ok(me.atime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
				});
				reg.add_method("permissions", |lua, _me, ()| {
					warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
					Ok(
						#[cfg(unix)]
						Some(yazi_shared::fs::permissions(_me.mode, _me.is_dummy())),
						#[cfg(windows)]
						None::<String>,
					)
				});
			}
		})?;

		Ok(())
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		#[inline]
		fn parse_time(f: Option<f64>) -> mlua::Result<Option<SystemTime>> {
			Ok(match f {
				Some(n) if n >= 0.0 => Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(n)),
				Some(n) => Err(format!("Invalid timestamp: {n}").into_lua_err())?,
				None => None,
			})
		}

		lua.globals().raw_set(
			"Cha",
			lua.create_function(|lua, t: Table| {
				let kind =
					ChaKind::from_bits(t.raw_get("kind")?).ok_or_else(|| "Invalid kind".into_lua_err())?;

				Self::cast(lua, yazi_shared::fs::Cha {
					kind,
					len: t.raw_get("len").unwrap_or_default(),
					atime: parse_time(t.raw_get("atime").ok())?,
					btime: parse_time(t.raw_get("btime").ok())?,
					#[cfg(unix)]
					ctime: parse_time(t.raw_get("ctime").ok())?,
					mtime: parse_time(t.raw_get("mtime").ok())?,
					#[cfg(unix)]
					mode: t.raw_get("mode").unwrap_or_default(),
					#[cfg(unix)]
					uid: t.raw_get("uid").unwrap_or_default(),
					#[cfg(unix)]
					gid: t.raw_get("gid").unwrap_or_default(),
					#[cfg(unix)]
					nlink: t.raw_get("nlink").unwrap_or_default(),
				})
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::Cha>> Cast<T> for Cha {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
