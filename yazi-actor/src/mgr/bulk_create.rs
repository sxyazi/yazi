use std::{fmt::{self, Display}, io::{self, Read, Write}, path::{MAIN_SEPARATOR, Path}};

use anyhow::{Result, anyhow};
use scopeguard::defer;
use yazi_binding::Permit;
use yazi_config::{YAZI, opener::OpenerRuleArc};
use yazi_fs::{FilesOp, Splatter, engine::{Engine, local::Local}};
use yazi_macro::{succ, writef};
use yazi_parser::VoidForm;
use yazi_proxy::TasksProxy;
use yazi_scheduler::{AppProxy, NotifyProxy, process::ShellOpt};
use yazi_shared::{data::Data, strand::Strand, url::{UrlBuf, UrlLike}};
use yazi_shim::path::CROSS_SEPARATOR;
use yazi_term::YIELD_TO_SUBPROCESS;
use yazi_tty::{TTY, sequence::EraseScreen};
use yazi_vfs::engine;
use yazi_watcher::WATCHER;

use crate::{Actor, Ctx};
pub struct BulkCreate;
impl Actor for BulkCreate {
	type Form = VoidForm;

	const NAME: &str = "bulk_create";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let Some(opener) = Self::opener() else {
			succ!(NotifyProxy::push_warn("Bulk create", "No text opener found"));
		};

		let cwd = cx.cwd().clone();
		tokio::spawn(async move {
			let tmp = YAZI.preview.tmpfile("bulk-create");
			let file = engine::create_new(&tmp).await?.file().await?;

			defer! {
				let tmp = tmp.clone();
				tokio::spawn(async move {
					Local::regular(&tmp).remove_file().await
				});
			}

			TasksProxy::process_exec(ShellOpt {
				cwd:    cwd.clone(),
				cmd:    Splatter::new(&[file]).splat(&opener.run),
				block:  opener.block,
				orphan: opener.orphan,
			})
			.await;

			let _permit = Permit::new(YIELD_TO_SUBPROCESS.acquire().await.unwrap(), AppProxy::resume());
			AppProxy::stop().await;

			let content = Local::regular(&tmp).read_to_string().await?;
			Self::r#do(cwd, content.lines().filter_map(Entry::parse).collect()).await
		});
		succ!()
	}
}

impl BulkCreate {
	async fn r#do(cwd: UrlBuf, todo: Vec<Entry<'_>>) -> Result<()> {
		writef!(TTY.writer(), "{EraseScreen}\n")?;
		if todo.is_empty() {
			return Ok(());
		} else if !Self::ask_continue(&todo, None)? {
			return Ok(()); // TODO: support `bulk_exit`?
		}

		let _permit = WATCHER.acquire().await.unwrap();
		let (mut failed, mut succeeded) = (vec![], Vec::with_capacity(todo.len()));
		for entry in todo {
			let Ok(dist) = cwd.try_join(entry.path) else {
				failed.push((entry, anyhow!("Invalid path")));
				continue;
			};

			let result: io::Result<()> = if entry.is_dir {
				engine::create_dir_all(&dist).await
			} else if let Some(parent) = dist.parent() {
				engine::create_dir_all(parent).await.ok();
				engine::create_new(&dist).await.map(|_| ())
			} else {
				Err(io::Error::other("No parent directory"))
			};

			if let Err(e) = result {
				failed.push((entry, e.into()));
			} else if let Ok(f) = engine::file(dist).await {
				succeeded.push(f);
			} else {
				failed.push((entry, anyhow!("Failed to retrieve file info")));
			}
		}

		if !succeeded.is_empty() {
			// err!(Pubsub::pub_after_bulk_create(it));  // FIXME
			FilesOp::create(succeeded);
		}
		drop(_permit);

		if !failed.is_empty() {
			Self::output_failed(failed).await?;
		}
		Ok(())
	}

	fn opener() -> Option<OpenerRuleArc> {
		YAZI
			.open
			.match_dummy(Path::new("bulk-create.txt"), "text/plain")
			.and_then(|r| YAZI.opener.block(&r))
	}

	fn ask_continue(todo: &[Entry], decision: Option<bool>) -> Result<bool> {
		if let Some(decision) = decision {
			return Ok(decision);
		}

		{
			let mut w = TTY.lockout();
			for entry in todo {
				writeln!(w, "{entry}")?;
			}
			write!(w, "Continue to create? (y/N): ")?;
			w.flush()?;
		}

		let mut buf = [0; 10];
		_ = TTY.reader().read(&mut buf)?;
		Ok(buf[0] == b'y' || buf[0] == b'Y')
	}

	async fn output_failed(failed: Vec<(Entry<'_>, anyhow::Error)>) -> Result<()> {
		let mut stdout = TTY.lockout();
		writeln!(stdout, "{EraseScreen}")?;

		writeln!(stdout, "Failed to create:")?;
		for (entry, err) in failed {
			writeln!(stdout, "{entry}: {err}")?;
		}
		writeln!(stdout, "\nPress ENTER to exit")?;

		stdout.flush()?;
		TTY.reader().read_exact(&mut [0])?;
		Ok(())
	}
}

// --- Entry
struct Entry<'a> {
	path:   Strand<'a>,
	is_dir: bool,
}

impl<'a> Entry<'a> {
	fn parse(s: &'a str) -> Option<Self> {
		let (path, is_dir) = match s.strip_suffix(CROSS_SEPARATOR) {
			Some(p) => (p, true),
			None => (s, false),
		};

		(!path.is_empty()).then_some(Self { path: path.into(), is_dir })
	}
}

impl Display for Entry<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.is_dir {
			write!(f, "{}{MAIN_SEPARATOR}", self.path.display())
		} else {
			self.path.display().fmt(f)
		}
	}
}
