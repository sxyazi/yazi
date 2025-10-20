use std::sync::Arc;

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

		// Get the appropriate context string for matching exclude rules
		// For search directories, use "search://**" as the context
		let cwd = cx.cwd();
		let cwd_str = if cwd.is_search() {
			"search://**".to_string()
		} else {
			cwd.as_path().map(|p| p.display().to_string()).unwrap_or_default()
		};
		
		let exclude_patterns = YAZI.files.excludes_for_context(&cwd_str);

		// Create glob matcher function for compiled patterns
		let glob_matcher = {
			let context = cwd_str.clone();
			Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context))
		};

		// If gitignores is disabled but we have exclude patterns, apply them
		if !gitignores && !exclude_patterns.is_empty() {
			// Load ignore filter from exclude patterns only
			let ignore_filter = if let Some(path) = cx.cwd().as_path() {
				IgnoreFilter::from_patterns(path, &exclude_patterns, Some(glob_matcher.clone()))
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
				let hovered_str = if h.url.is_search() {
					"search://**".to_string()
				} else {
					h.url.as_path().map(|p| p.display().to_string()).unwrap_or_default()
				};
				let hovered_excludes = YAZI.files.excludes_for_context(&hovered_str);
				let hovered_matcher = {
					let context = hovered_str.clone();
					Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context))
				};
				let hovered_filter = if let Some(path) = h.url.as_path() {
					IgnoreFilter::from_patterns(path, &hovered_excludes, Some(hovered_matcher))
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
			IgnoreFilter::from_dir(path, &exclude_patterns, gitignores, Some(glob_matcher.clone()))
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
			let hovered_str = if h.url.is_search() {
				"search://**".to_string()
			} else {
				h.url.as_path().map(|p| p.display().to_string()).unwrap_or_default()
			};
			let hovered_excludes = YAZI.files.excludes_for_context(&hovered_str);
			let hovered_matcher = {
				let context = hovered_str.clone();
				Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context))
			};
			let hovered_filter = if let Some(path) = h.url.as_path() {
				IgnoreFilter::from_dir(path, &hovered_excludes, gitignores, Some(hovered_matcher))
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
