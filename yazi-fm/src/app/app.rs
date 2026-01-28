use std::{sync::atomic::Ordering, time::{Duration, Instant}};

use anyhow::Result;
use tokio::{select, time::sleep};
use yazi_actor::Ctx;
use yazi_core::Core;
use yazi_macro::act;
use yazi_shared::{data::Data, event::{Event, NEED_RENDER}};
use yazi_term::Term;

use crate::{Dispatcher, Signals};

pub(crate) struct App {
	pub(crate) core:    Core,
	pub(crate) term:    Option<Term>,
	pub(crate) signals: Signals,
}

impl App {
	pub(crate) async fn serve() -> Result<()> {
		let term = Term::start()?;
		let (mut rx, signals) = (Event::take(), Signals::start()?);

		let mut app = Self { core: Core::make(), term: Some(term), signals };
		app.bootstrap()?;

		let mut events = Vec::with_capacity(50);
		let (mut timeout, mut need_render, mut last_render) = (None, 0, Instant::now());
		macro_rules! drain_events {
			() => {
				for event in events.drain(..) {
					Dispatcher::new(&mut app).dispatch(event)?;

					need_render = NEED_RENDER.load(Ordering::Relaxed);
					if need_render == 0 {
						continue;
					}

					timeout = Duration::from_millis(10).checked_sub(last_render.elapsed());
					if timeout.is_none() {
						app.render(need_render == 2)?;
						last_render = Instant::now();
					}
				}
			};
		}

		loop {
			if let Some(t) = timeout.take() {
				select! {
					_ = sleep(t) => {
						app.render(need_render == 2)?;
						last_render = Instant::now();
					}
					n = rx.recv_many(&mut events, 50) => {
						if n == 0 { break }
						drain_events!();
					}
				}
			} else if rx.recv_many(&mut events, 50).await != 0 {
				drain_events!();
			} else {
				break;
			}
		}
		Ok(())
	}

	fn bootstrap(&mut self) -> anyhow::Result<Data> {
		let cx = &mut Ctx::active(&mut self.core, &mut self.term);
		act!(app:bootstrap, cx)?;

		self.render(false)
	}
}
