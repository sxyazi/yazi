use std::{borrow::Cow, collections::{HashMap, HashSet}, ffi::{OsStr, OsString}, io::{stderr, BufWriter, Write}, path::PathBuf};

use anyhow::{Result, anyhow};
use scopeguard::defer;
use tokio::{io::{AsyncReadExt, AsyncWriteExt, stdin}};
use tokio::fs;
use yazi_config::{OPEN, PREVIEW};
use yazi_dds::Pubsub;
use yazi_fs::{max_common_root, maybe_exists, ok_or_not_found, paths_to_same_file, realname, File, FilesOp};
use yazi_proxy::{AppProxy, HIDER, TasksProxy, WATCHER};
use yazi_shared::{terminal_clear, url::{Url, UrnBuf}};

use crate::manager:: Manager;

impl Manager {
	pub(super) fn bulk_create(&self) {
		let Some(opener) = OPEN.block_opener("bulk-create.txt", "text/plain") else {
			return AppProxy::notify_warn("Bulk create", "No text opener found");
		};

		let cwd = self.cwd().clone();
		tokio::spawn(async move {
			let tmp = PREVIEW.tmpfile("bulk");
			defer! { tokio::spawn(fs::remove_file(tmp.clone())); }
			TasksProxy::process_exec(Cow::Borrowed(opener), cwd, vec![
				OsString::new(),
				tmp.to_owned().into(),
			])
			.await;

			let _permit = HIDER.acquire().await.unwrap();
			defer!(AppProxy::resume());
			AppProxy::stop().await;

			let new: Vec<_> = fs::read_to_string(&tmp).await?.lines().map(Url::from).collect();			
			Self::bulk_create_do( new).await
		});
	}

	async fn bulk_create_do(new:Vec<Url>) -> Result<()> {
		terminal_clear(&mut stderr())?;
		if new.is_empty() {
			return Ok(());
		}

		{
			let mut stderr = BufWriter::new(stderr().lock());
			for n in &new {
				if n.to_str().unwrap().ends_with('/') || n.to_str().unwrap().ends_with('\\') {
					writeln!(stderr, "create new dir-> {}", n.display())?;
				}else{
					writeln!(stderr, "create new file-> {}", n.display())?;
				}
			}
			write!(stderr, "Continue to create? (y/N): ")?;
			stderr.flush()?;
		}

		let mut buf = [0; 10];
		_ = stdin().read(&mut buf).await?;
		if buf[0] != b'y' && buf[0] != b'Y' {
			return Ok(());
		}

		let permit = WATCHER.acquire().await.unwrap();
		let (mut failed, mut succeeded) = (Vec::new(), HashMap::with_capacity(new.len()));
        for n in new {
            let Some(parent) = n.parent_url() else { return Ok(()) };
            let dir = n.to_str().unwrap().ends_with('/') || n.to_str().unwrap().ends_with('\\');
            
            if dir {
                if let Err(e) = fs::create_dir_all(&n).await {
                    failed.push((PathBuf::new(), n.into(), e.into()));
                } else if let Ok(f) = File::from(n.clone()).await {
                    succeeded.insert(n.clone(), f);
                } else {
                    failed.push((PathBuf::new(), n, anyhow!("Failed to retrieve file info")));
                }
            } else {
                fs::create_dir_all(&parent).await.ok();
                if let Some(real) = realname(&n).await {
                    if let Err(e) = ok_or_not_found(fs::remove_file(&n).await) {
                        failed.push((PathBuf::new(), n.into(), e.into()));
                        continue;
                    }
                    FilesOp::Deleting(parent.clone(), HashSet::from_iter([UrnBuf::from(real)])).emit();
                }
                
                if let Err(e) = fs::File::create(&n).await {
                    failed.push((PathBuf::new(), n.into(), e.into()));
                } else if let Ok(f) = File::from(n.clone()).await {
                    succeeded.insert(n.clone(), f);
                } else {
                    failed.push((PathBuf::new(), n.into(), anyhow!("Failed to retrieve file info")));
                }
            }
        }
		
		

		if !succeeded.is_empty() {
			Pubsub::pub_from_bulk(succeeded.iter().map(|(o, n)| (o, &n.url)).collect());
            FilesOp::create(succeeded.into_values().collect());
		}
		drop(permit);

		if !failed.is_empty() {
			Self::output_failed_for_bulk_create(failed).await?;
		}
		Ok(())
	}

	async fn output_failed_for_bulk_create(failed: Vec<(PathBuf, Url, anyhow::Error)>) -> Result<()> {
		terminal_clear(&mut stderr())?;

		{
			let mut stderr = BufWriter::new(stderr().lock());
			writeln!(stderr, "Failed to create:")?;
			for (o, n, e) in failed {
				writeln!(stderr, "{} -> {}: {e}", o.display(), n.display())?;
			}
			writeln!(stderr, "\nPress ENTER to exit")?;
			stderr.flush()?;
		}

		stdin().read_exact(&mut [0]).await?;
		Ok(())
	}
}
