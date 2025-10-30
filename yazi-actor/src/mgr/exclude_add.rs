use std::sync::Arc;

use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use yazi_config::YAZI;
use yazi_core::tab::Folder;
use yazi_fs::{FolderStage, IgnoreFilter};
use yazi_macro::{act, render, render_and, succ};
use yazi_parser::mgr::ExcludeAddOpt;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct ExcludeAdd;

impl Actor for ExcludeAdd {
	type Options = ExcludeAddOpt;

	const NAME: &str = "exclude_add";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		if opt.patterns.is_empty() {
			succ!();
		}

		// Get the appropriate context string for matching exclude rules
		let cwd = cx.cwd();
		let cwd_str = if cwd.is_search() {
			"search://**".to_string()
		} else {
			cwd.as_path().map(|p| p.display().to_string()).unwrap_or_default()
		};

		// Get existing patterns from config
		let config_patterns = YAZI.files.excludes_for_context(&cwd_str);

		// Merge plugin patterns with config patterns
		// Config patterns come last so they can override plugin patterns with negation
		let mut all_patterns = opt.patterns.clone();
		all_patterns.extend(config_patterns);

		// Compile glob patterns
		let mut ignores_builder = GlobSetBuilder::new();
		let mut whitelists_builder = GlobSetBuilder::new();

		for pattern in &all_patterns {
			if let Some(negated) = pattern.strip_prefix('!') {
				// Negation pattern - this is a whitelist
				if let Ok(glob) = Glob::new(negated) {
					whitelists_builder.add(glob);
				}
			} else {
				// Regular ignore pattern
				if let Ok(glob) = Glob::new(pattern) {
					ignores_builder.add(glob);
				}
			}
		}

		let ignores = ignores_builder.build().ok();
		let whitelists = whitelists_builder.build().ok();

		// Create glob matcher function for compiled patterns
		let glob_matcher: Option<Arc<dyn Fn(&std::path::Path) -> Option<bool> + Send + Sync>> =
			if ignores.is_some() || whitelists.is_some() {
				let context = cwd_str.clone();
				Some(Arc::new(move |path: &std::path::Path| {
					// First check config patterns (for user overrides/negation)
					if let Some(result) = YAZI.files.matches_path(path, &context) {
						return Some(result);
					}

					// For absolute paths, try both the full path and relative components
					let paths_to_check: Vec<&std::path::Path> = if path.is_absolute() {
						let mut paths = vec![path];
						if let Some(components) = path.to_str() {
							for (i, _) in components.match_indices('/').skip(1) {
								if let Some(subpath) = components.get(i + 1..) {
									paths.push(std::path::Path::new(subpath));
								}
							}
						}
						paths
					} else {
						vec![path]
					};

					// Check whitelist first (negation takes precedence)
					if let Some(ref wl) = whitelists {
						for p in &paths_to_check {
							if wl.is_match(p) {
								return Some(false); // Explicitly NOT ignored
							}
						}
					}

					// Check ignore patterns
					if let Some(ref ig) = ignores {
						for p in &paths_to_check {
							if ig.is_match(p) {
								return Some(true); // Should be ignored
							}
						}
					}

					None
				}))
			} else {
				None
			};

		// Load ignore filter with merged patterns
		let ignore_filter = IgnoreFilter::from_patterns(glob_matcher.clone());

		let hovered = cx.hovered().map(|f| f.urn().to_owned());
		let apply = |f: &mut Folder, filter: Option<IgnoreFilter>| {
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

		// Apply to hovered folder
		if let Some(h) = cx.hovered_folder_mut() {
			let hovered_str = if h.url.is_search() {
				"search://**".to_string()
			} else {
				h.url.as_path().map(|p| p.display().to_string()).unwrap_or_default()
			};

			let hovered_config_patterns = YAZI.files.excludes_for_context(&hovered_str);
			let mut hovered_all_patterns = opt.patterns;
			hovered_all_patterns.extend(hovered_config_patterns);

			let hovered_matcher: Option<Arc<dyn Fn(&std::path::Path) -> Option<bool> + Send + Sync>> =
				if !hovered_all_patterns.is_empty() {
					let context = hovered_str.clone();
					let patterns = hovered_all_patterns.clone();
					Some(Arc::new(move |path: &std::path::Path| {
						if let Some(result) = YAZI.files.matches_path(path, &context) {
							return Some(result);
						}
						for pattern in &patterns {
							if let Some(negated) = pattern.strip_prefix('!') {
								if path.to_str().map_or(false, |p| p.contains(negated)) {
									return Some(false);
								}
							} else if path.to_str().map_or(false, |p| p.contains(pattern)) {
								return Some(true);
							}
						}
						None
					}))
				} else {
					None
				};

			let hovered_filter = IgnoreFilter::from_patterns(hovered_matcher);

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
