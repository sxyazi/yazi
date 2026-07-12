use std::io;

use mlua::{IntoLua, Lua, Value};
use strum::AsRefStr;
use yazi_binding::MpscTx;
use yazi_fs::engine::{Attrs, Demand};
use yazi_shared::{path::PathBufDyn, url::UrlBuf};

#[derive(AsRefStr)]
#[strum(serialize_all = "PascalCase")]
pub enum ProviderJob {
	Capabilities,
	Absolute {
		url: UrlBuf,
	},
	Canonicalize {
		url: UrlBuf,
	},
	Casefold {
		url: UrlBuf,
	},
	SymlinkMetadata {
		url: UrlBuf,
	},
	Metadata {
		url: UrlBuf,
	},
	ReadDir {
		url: UrlBuf,
	},
	Open {
		url:    UrlBuf,
		attrs:  Attrs,
		demand: Demand,
	},
	CreateDir {
		url: UrlBuf,
	},
	HardLink {
		from: UrlBuf,
		to:   PathBufDyn,
	},
	ReadLink {
		url: UrlBuf,
	},
	RemoveDir {
		url: UrlBuf,
	},
	RemoveFile {
		url: UrlBuf,
	},
	Rename {
		from: UrlBuf,
		to:   PathBufDyn,
	},
	Symlink {
		original: Vec<u8>,
		url:      UrlBuf,
		is_dir:   bool,
	},
	Trash {
		url: UrlBuf,
	},
	Read {
		url:    UrlBuf,
		offset: u64,
		len:    usize,
	},
	Write {
		url:    UrlBuf,
		offset: u64,
		bytes:  Vec<u8>,
	},
	Copy {
		from:  UrlBuf,
		to:    PathBufDyn,
		attrs: Attrs,
	},
	CopyProgressive {
		from:  UrlBuf,
		to:    PathBufDyn,
		attrs: Attrs,
		tx:    MpscTx<u64, io::Result<u64>>,
	},
	SetLen {
		url:  UrlBuf,
		size: u64,
	},
	SetAttrs {
		url:   UrlBuf,
		attrs: Attrs,
	},
}

impl IntoLua for ProviderJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let t = lua.create_table()?;
		t.raw_set("op", self.as_ref())?;

		match self {
			Self::Capabilities => {}

			Self::Absolute { url }
			| Self::Canonicalize { url }
			| Self::Casefold { url }
			| Self::SymlinkMetadata { url }
			| Self::Metadata { url }
			| Self::ReadDir { url }
			| Self::CreateDir { url }
			| Self::ReadLink { url }
			| Self::RemoveDir { url }
			| Self::RemoveFile { url }
			| Self::Trash { url } => t.raw_set("url", url)?,

			Self::Open { url, attrs, demand } => {
				t.raw_set("url", url)?;
				t.raw_set("attrs", attrs)?;
				t.raw_set("demand", demand)?;
			}
			Self::HardLink { from, to } | Self::Rename { from, to } => {
				t.raw_set("from", from)?;
				t.raw_set("to", to)?;
			}
			Self::Symlink { original, url, is_dir } => {
				t.raw_set("original", lua.create_external_string(original)?)?;
				t.raw_set("url", url)?;
				t.raw_set("is_dir", is_dir)?;
			}
			Self::Read { url, offset, len } => {
				t.raw_set("url", url)?;
				t.raw_set("offset", offset)?;
				t.raw_set("len", len)?;
			}
			Self::Write { url, offset, bytes } => {
				t.raw_set("url", url)?;
				t.raw_set("offset", offset)?;
				t.raw_set("bytes", lua.create_external_string(bytes)?)?;
			}
			Self::Copy { from, to, attrs } => {
				t.raw_set("from", from)?;
				t.raw_set("to", to)?;
				t.raw_set("attrs", attrs)?;
			}
			Self::CopyProgressive { from, to, attrs, tx } => {
				t.raw_set("from", from)?;
				t.raw_set("to", to)?;
				t.raw_set("attrs", attrs)?;
				t.raw_set("tx", tx)?;
			}
			Self::SetLen { url, size } => {
				t.raw_set("url", url)?;
				t.raw_set("size", size)?;
			}
			Self::SetAttrs { url, attrs } => {
				t.raw_set("url", url)?;
				t.raw_set("attrs", attrs)?;
			}
		}

		t.into_lua(lua)
	}
}
