use anyhow::Result;
use yazi_config::YAZI;
use yazi_core::tab::Folder;
use yazi_fs::{FolderStage, IgnoreFilter};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::VoidOpt;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Ignore;

impl Actor for Ignore {
	type Options = VoidOpt;

	const NAME: &str = "ignore";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let gitignores = YAZI.files.gitignores;

		// Get exclude patterns for the current context
		// Use path string for context matching
		let cwd_str = cx.cwd().as_path().map(|p| p.display().to_string()).unwrap_or_default();
		let exclude_patterns = YAZI.files.excludes_for_context(&cwd_str);

		// If gitignores is disabled but we have exclude patterns, apply them
		if !gitignores && !exclude_patterns.is_empty() {
			// Load ignore filter from exclude patterns only
			let ignore_filter = if let Some(path) = cx.cwd().as_path() {
				IgnoreFilter::from_patterns(path, &exclude_patterns)
			} else {
				None
			};

			let hovered = cx.hovered().map(|f| f.urn().to_owned());
			let apply = |f: &mut Folder, filter: Option<IgnoreFilter>| {
				// Always set the filter, even when loading
				let changed = f.files.set_ignore_filter(filter);
				if f.stage == FolderStage::Loading {
					render!();
					false
				} else {
					render_and!(changed && f.files.catchup_revision())
				}
			};

			// Apply to CWD
			if apply(cx.current_mut(), ignore_filter.clone()) {
				act!(mgr:hover, cx)?;
				act!(mgr:update_paged, cx)?;
			}

			// Apply to hovered
			if let Some(h) = cx.hovered_folder_mut() {
				let hovered_str = h.url.as_path().map(|p| p.display().to_string()).unwrap_or_default();
				let hovered_excludes = YAZI.files.excludes_for_context(&hovered_str);
				let hovered_filter = if let Some(path) = h.url.as_path() {
					IgnoreFilter::from_patterns(path, &hovered_excludes)
				} else {
					None
				};

				if apply(h, hovered_filter) {
					render!(h.repos(None));
					act!(mgr:peek, cx, true)?;
				} else if hovered.as_deref() != cx.hovered().map(|f| f.urn()) {
					act!(mgr:peek, cx)?;
					act!(mgr:watch, cx)?;
				}
			}

			succ!();
		}

		// If gitignores is disabled and no exclude patterns, remove any ignore filter
		if !gitignores {
			let hovered = cx.hovered().map(|f| f.urn().to_owned());
			let apply = |f: &mut Folder| {
				// Always clear the filter, even when loading
				let changed = f.files.set_ignore_filter(None);
				if f.stage == FolderStage::Loading {
					render!();
					false
				} else {
					render_and!(changed && f.files.catchup_revision())
				}
			};

			// Apply to CWD and parent
			if let (a, Some(b)) = (apply(cx.current_mut()), cx.parent_mut().map(apply))
				&& (a | b)
			{
				act!(mgr:hover, cx)?;
				act!(mgr:update_paged, cx)?;
			}

			// Apply to hovered
			if let Some(h) = cx.hovered_folder_mut()
				&& apply(h)
			{
				render!(h.repos(None));
				act!(mgr:peek, cx, true)?;
			} else if hovered.as_deref() != cx.hovered().map(|f| f.urn()) {
				act!(mgr:peek, cx)?;
				act!(mgr:watch, cx)?;
			}

			succ!();
		}

		// Load ignore filter from the current directory
		let ignore_filter = if let Some(path) = cx.cwd().as_path() {
			IgnoreFilter::from_dir(path, &exclude_patterns, gitignores)
		} else {
			None
		};

		let hovered = cx.hovered().map(|f| f.urn().to_owned());
		let apply = |f: &mut Folder, filter: Option<IgnoreFilter>| {
			// Always set the filter, even when loading, so files are filtered as they
			// arrive
			let changed = f.files.set_ignore_filter(filter);
			if f.stage == FolderStage::Loading {
				render!();
				false
			} else {
				render_and!(changed && f.files.catchup_revision())
			}
		};

		// Apply to CWD
		if apply(cx.current_mut(), ignore_filter.clone()) {
			act!(mgr:hover, cx)?;
			act!(mgr:update_paged, cx)?;
		}

		// Apply to parent (they should use their own ignore file, so we don't apply the
		// same filter) Parent folders will be updated when they become the current
		// directory

		// Apply to hovered
		if let Some(h) = cx.hovered_folder_mut() {
			// Load ignore filter for hovered directory if it's a directory
			let hovered_str = h.url.as_path().map(|p| p.display().to_string()).unwrap_or_default();
			let hovered_excludes = YAZI.files.excludes_for_context(&hovered_str);
			let hovered_filter = if let Some(path) = h.url.as_path() {
				IgnoreFilter::from_dir(path, &hovered_excludes, gitignores)
			} else {
				None
			};

			if apply(h, hovered_filter) {
				render!(h.repos(None));
				act!(mgr:peek, cx, true)?;
			} else if hovered.as_deref() != cx.hovered().map(|f| f.urn()) {
				act!(mgr:peek, cx)?;
				act!(mgr:watch, cx)?;
			}
		}

		succ!();
	}
}
