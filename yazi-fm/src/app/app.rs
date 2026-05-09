use std::{sync::atomic::Ordering, time::{Duration, Instant}};

use anyhow::Result;
use tokio::{select, sync::mpsc, time::sleep};
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

	need_render:            u8,
	pub(crate) last_render: Instant,
	next_render:            Option<Duration>,
}

impl App {
	fn make(term: Term, signals: Signals) -> Result<Self> {
		Ok(Self {
			core: Core::make(),
			term: Some(term),
			signals,

			need_render: 0,
			last_render: Instant::now(),
			next_render: None,
		})
	}

	pub(crate) async fn serve() -> Result<()> {
		let term = Term::start()?;
		let (mut rx, signals) = (Event::take(), Signals::start()?);

		let mut app = Self::make(term, signals)?;
		app.bootstrap()?;

		loop {
			if let Some(t) = app.next_render.take() {
				select! {
					_ = sleep(t) => {
						app.render(app.need_render == 2)?;
					}
					r = app.drain(&mut rx) => if !r? {
						break;
					}
				}
			} else if !app.drain(&mut rx).await? {
				break;
			}
		}
		Ok(())
	}

	fn bootstrap(&mut self) -> Result<Data> {
		let cx = &mut Ctx::active(&mut self.core, &mut self.term);
		act!(app:bootstrap, cx)?;

		self.render(false)
	}

	async fn drain(&mut self, rx: &mut mpsc::UnboundedReceiver<Event>) -> Result<bool> {
		let Some(event) = rx.recv().await else {
			return Ok(false);
		};

		self.dispatch(event)?;
		while let Ok(e) = rx.try_recv() {
			self.dispatch(e)?;
		}

		Ok(true)
	}

	fn dispatch(&mut self, event: Event) -> Result<()> {
		Dispatcher::new(self).dispatch(event);

		self.need_render = NEED_RENDER.load(Ordering::Relaxed);
		if self.need_render == 0 {
			return Ok(());
		}

		self.next_render = Duration::from_millis(10).checked_sub(self.last_render.elapsed());
		if self.next_render.is_none() {
			self.render(self.need_render == 2)?;
		}

		Ok(())
	}
}
