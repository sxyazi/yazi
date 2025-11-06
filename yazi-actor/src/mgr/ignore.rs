use std::sync::Arc;

use anyhow::Result;
use yazi_config::YAZI;
use yazi_core::tab::Folder;
use yazi_fs::{FolderStage, IgnoreFilter};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::VoidOpt;
use yazi_shared::{data::Data, path::PathLike, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Ignore;

impl Actor for Ignore {
	type Options = VoidOpt;

	const NAME: &str = "ignore";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
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
		let glob_matcher: Option<Arc<dyn Fn(&std::path::Path) -> Option<bool> + Send + Sync>> =
			if !exclude_patterns.is_empty() {
				let context = cwd_str.clone();
				Some(Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context)))
			} else {
				None
			};

		// Load ignore filter from exclude patterns
		let ignore_filter = IgnoreFilter::from_patterns(glob_matcher.clone());

		let hovered = cx.hovered().map(|f| f.urn().owned());
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

		// Apply to CWD and parent
		let cwd_changed = apply(cx.current_mut(), ignore_filter.clone());

		let parent_changed = if let Some(p) = cx.parent_mut() {
			let parent_str = if p.url.is_search() {
				"search://**".to_string()
			} else {
				p.url.as_path().map(|p| p.display().to_string()).unwrap_or_default()
			};

			let parent_excludes = YAZI.files.excludes_for_context(&parent_str);
			let parent_filter = if !parent_excludes.is_empty() {
				let context = parent_str.clone();
				let matcher: Option<Arc<dyn Fn(&std::path::Path) -> Option<bool> + Send + Sync>> =
					Some(Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context)));
				IgnoreFilter::from_patterns(matcher)
			} else {
				IgnoreFilter::from_patterns(None)
			};

			apply(p, parent_filter)
		} else {
			false
		};

		if cwd_changed || parent_changed {
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
			let hovered_matcher: Option<Arc<dyn Fn(&std::path::Path) -> Option<bool> + Send + Sync>> =
				if !hovered_excludes.is_empty() {
					let context = hovered_str.clone();
					Some(Arc::new(move |path: &std::path::Path| YAZI.files.matches_path(path, &context)))
				} else {
					None
				};
			let hovered_filter = IgnoreFilter::from_patterns(hovered_matcher);

			if apply(h, hovered_filter) {
				render!(h.repos(None));
				act!(mgr:peek, cx, true)?;
			} else if cx.hovered().map(|f| f.urn()) != hovered.as_ref().map(Into::into) {
				act!(mgr:peek, cx)?;
				act!(mgr:watch, cx)?;
			}
		}

		succ!();
	}
}
