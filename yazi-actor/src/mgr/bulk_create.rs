use crate::{Actor, Ctx};
use anyhow::{Result, anyhow, bail};
use scopeguard::defer;
use std::{
	io::{Read, Write},
	path::Path,
};
use yazi_binding::Permit;
use yazi_config::{YAZI, opener::OpenerRule};
use yazi_fs::{
	File, FilesOp, Splatter,
	provider::{
		Provider,
		local::{Gate, Local},
	},
};
use yazi_macro::{ok_or_not_found, succ};
use yazi_parser::VoidOpt;
use yazi_proxy::{AppProxy, MgrProxy, NotifyProxy, TasksProxy};
use yazi_shared::{
	data::Data,
	terminal_clear,
	url::{AsUrl, UrlBuf, UrlCow, UrlLike},
};
use yazi_term::YIELD_TO_SUBPROCESS;
use yazi_tty::TTY;
use yazi_vfs::{VfsFile, maybe_exists, provider};
use yazi_watcher::WATCHER;
pub struct BulkCreate;
impl Actor for BulkCreate {
	type Options = VoidOpt;
	const NAME: &str = "bulk_create";
	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let Some(opener) = Self::opener() else {
			succ!(NotifyProxy::push_warn("Bulk create", "No text opener found"));
		};
		let cwd = cx.cwd().clone();
		tokio::spawn(async move {
			let tmp = YAZI.preview.tmpfile("bulk");
			_ = Gate::default().write(true).create_new(true).open(&tmp).await?;
			defer! {
				let tmp = tmp.clone();
				tokio::spawn(async move {
					Local::regular(&tmp).remove_file().await
				});
			}
			TasksProxy::process_exec(
				cwd.clone().into(),
				Splatter::new(&[UrlCow::default(), tmp.as_url().into()]).splat(&opener.run),
				vec![UrlCow::default(), UrlBuf::from(&tmp).into()],
				opener.block,
				opener.orphan,
			)
			.await;
			let _permit = Permit::new(YIELD_TO_SUBPROCESS.acquire().await.unwrap(), AppProxy::resume());
			AppProxy::stop().await;
			let todo: Vec<_> =
				Local::regular(&tmp).read_to_string().await?.lines().filter_map(Entry::parse).collect();
			Self::r#do(cwd, todo).await
		});
		succ!()
	}
}
impl BulkCreate {
	async fn r#do(cwd: UrlBuf, todo: Vec<Entry>) -> Result<()> {
		terminal_clear(TTY.writer())?;
		if todo.is_empty() {
			return Ok(());
		}
		{
			let mut w = TTY.lockout();
			for entry in &todo {
				writeln!(w, "{}", entry.name)?;
			}
			write!(w, "Continue to create? (y/N): ")?;
			w.flush()?;
		}
		let mut buf = [0; 10];
		_ = TTY.reader().read(&mut buf)?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}
		let _permit = WATCHER.acquire().await.unwrap();
		let mut failed = Vec::new();
		let mut reveal = None;
		for entry in todo {
			let Ok(new) = cwd.try_join(&entry.name) else {
				failed.push((entry, anyhow!("Invalid path")));
				continue;
			};
			if maybe_exists(&new).await {
				failed.push((entry, anyhow!("Destination already exists")));
				continue;
			}
			match Self::create_one(&new, entry.dir).await {
				Ok(real) => reveal = Some(real),
				Err(e) => failed.push((entry, e)),
			}
		}
		if let Some(url) = reveal {
			MgrProxy::reveal(&url);
		}
		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}
	fn opener() -> Option<&'static OpenerRule> {
		YAZI.opener.block(YAZI.open.all(Path::new("bulk-rename.txt"), "text/plain"))
	}
	async fn create_one(new: &UrlBuf, dir: bool) -> Result<UrlBuf> {
		if dir {
			provider::create_dir_all(new).await?;
		} else if let Ok(real) = provider::casefold(new).await
			&& let Some((parent, urn)) = real.pair()
		{
			ok_or_not_found!(provider::remove_file(new).await);
			FilesOp::Deleting(parent.into(), [urn.into()].into()).emit();
			provider::create(new).await?;
		} else if let Some(parent) = new.parent() {
			provider::create_dir_all(parent).await.ok();
			ok_or_not_found!(provider::remove_file(new).await);
			provider::create(new).await?;
		} else {
			bail!("Cannot create file at root");
		}
		if let Ok(real) = provider::casefold(new).await
			&& let Some((parent, urn)) = real.pair()
		{
			let file = File::new(&real).await?;
			FilesOp::Upserting(parent.into(), [(urn.into(), file)].into()).emit();
			Ok(real)
		} else {
			bail!("Failed to retrieve file info");
		}
	}
	async fn output_failed(failed: Vec<(Entry, anyhow::Error)>) -> Result<()> {
		let mut stdout = TTY.lockout();
		terminal_clear(&mut *stdout)?;
		writeln!(stdout, "Failed to create:")?;
		for (entry, err) in failed {
			writeln!(stdout, "{}: {err}", entry.name)?;
		}
		writeln!(stdout, "\nPress ENTER to exit")?;
		stdout.flush()?;
		TTY.reader().read_exact(&mut [0])?;
		Ok(())
	}
}
struct Entry {
	name: String,
	dir: bool,
}
impl Entry {
	fn parse(s: &str) -> Option<Self> {
		let s = s.trim();
		if s.is_empty() {
			return None;
		}
		Some(Self { dir: s.ends_with('/') || s.ends_with('\\'), name: s.to_owned() })
	}
}
