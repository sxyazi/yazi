use mlua::{Lua, Table};
use strum::AsRefStr;
use tokio::sync::mpsc;
use yazi_binding::MpscRx;
use yazi_fs::{engine::{Attrs, Demand}, file::File};
use yazi_shared::{id::Id, path::PathBufDyn, url::UrlBuf};

use super::{Handle, ProvideChunk, ProvidePeer};

#[derive(AsRefStr)]
#[strum(serialize_all = "PascalCase")]
pub enum ProvideOp {
	Capabilities,
	Reroute { url: UrlBuf },
	Absolute { url: UrlBuf },
	Canonicalize { url: UrlBuf },
	Casefold { url: UrlBuf },
	SymlinkMetadata { url: UrlBuf },
	Metadata { url: UrlBuf, handle: Option<Handle> },
	ReadDir { url: UrlBuf },
	Revalidate { file: File },
	File { url: UrlBuf, handle: Option<Handle> },
	Open { url: UrlBuf, attrs: Attrs, demand: Demand },
	CreateDir { url: UrlBuf },
	CreateDirAll { url: UrlBuf },
	CreateFile { url: UrlBuf },
	CreateFileNew { url: UrlBuf },
	HardLink { from: UrlBuf, to: PathBufDyn },
	ReadLink { url: UrlBuf },
	RemoveDir { url: UrlBuf },
	RemoveDirAll { url: UrlBuf },
	RemoveFile { url: UrlBuf },
	Rename { from: UrlBuf, to: PathBufDyn },
	Symlink { original: Vec<u8>, url: UrlBuf, is_dir: bool },
	Trash { url: UrlBuf },
	Read { url: UrlBuf, handle: Handle },
	Write { url: UrlBuf, handle: Handle, stream: mpsc::Receiver<ProvideChunk> },
	Close { url: UrlBuf, handle: Handle },
	CopyTo { from: UrlBuf, to: UrlBuf, peer: ProvidePeer, attrs: Attrs },
	CopyFrom { from: UrlBuf, to: UrlBuf, peer: ProvidePeer, attrs: Attrs },
	SetLen { url: UrlBuf, size: u64, handle: Handle },
	SetAttrs { url: UrlBuf, attrs: Attrs, handle: Option<Handle> },
}

impl ProvideOp {
	pub(super) fn into_table(self, lua: &Lua) -> mlua::Result<Table> {
		let t = lua.create_table()?;
		t.raw_set("op", self.as_ref())?;

		match self {
			Self::Capabilities => {}
			Self::Revalidate { file } => t.raw_set("file", file)?,

			Self::Reroute { url }
			| Self::Absolute { url }
			| Self::Canonicalize { url }
			| Self::Casefold { url }
			| Self::SymlinkMetadata { url }
			| Self::ReadDir { url }
			| Self::CreateDir { url }
			| Self::CreateDirAll { url }
			| Self::CreateFile { url }
			| Self::CreateFileNew { url }
			| Self::ReadLink { url }
			| Self::RemoveDir { url }
			| Self::RemoveDirAll { url }
			| Self::RemoveFile { url }
			| Self::Trash { url } => t.raw_set("url", url)?,

			Self::Metadata { url, handle } | Self::File { url, handle } => {
				t.raw_set("url", url)?;
				handle.map(|h| h.into_table(lua, &t)).transpose()?;
			}
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
			Self::Read { url, handle } => {
				t.raw_set("url", url)?;
				handle.into_table(lua, &t)?;
			}
			Self::Write { url, handle, stream } => {
				t.raw_set("url", url)?;
				t.raw_set("stream", MpscRx(stream))?;
				handle.into_table(lua, &t)?;
			}
			Self::Close { url, handle } => {
				t.raw_set("url", url)?;
				handle.into_table(lua, &t)?;
			}
			Self::CopyTo { from, to, peer, attrs } | Self::CopyFrom { from, to, peer, attrs } => {
				t.raw_set("from", from)?;
				t.raw_set("to", to)?;
				t.raw_set("peer", peer)?;
				t.raw_set("attrs", attrs)?;
			}
			Self::SetLen { url, size, handle } => {
				t.raw_set("url", url)?;
				t.raw_set("size", Id(size))?;
				handle.into_table(lua, &t)?;
			}
			Self::SetAttrs { url, attrs, handle } => {
				t.raw_set("url", url)?;
				t.raw_set("attrs", attrs)?;
				handle.map(|h| h.into_table(lua, &t)).transpose()?;
			}
		}

		Ok(t)
	}
}
