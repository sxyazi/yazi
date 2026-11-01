use std::time::Duration;

use tokio::{pin, select, sync::mpsc};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_macro::log_if_err;
use yazi_shared::url::UrlBuf;
use yazi_shim::cell::RoCell;

use crate::{Pubsub, ember::{EmberDuplicateItem, EmberMoveItem}};

static PUMP: RoCell<Pump> = RoCell::new();

pub struct Pump {
	duplicate_tx: mpsc::UnboundedSender<EmberDuplicateItem>,
	move_tx:      mpsc::UnboundedSender<EmberMoveItem>,
	trash_tx:     mpsc::UnboundedSender<UrlBuf>,
	delete_tx:    mpsc::UnboundedSender<UrlBuf>,
	download_tx:  mpsc::UnboundedSender<UrlBuf>,
	shutdown_tx:  mpsc::UnboundedSender<()>,
}

impl Pump {
	pub fn push_duplicate<U>(from: U, to: U)
	where
		U: Into<UrlBuf>,
	{
		PUMP.duplicate_tx.send(EmberDuplicateItem { from: from.into(), to: to.into() }).ok();
	}

	pub fn push_move<U>(from: U, to: U)
	where
		U: Into<UrlBuf>,
	{
		PUMP.move_tx.send(EmberMoveItem { from: from.into(), to: to.into() }).ok();
	}

	pub fn push_trash<U>(target: U)
	where
		U: Into<UrlBuf>,
	{
		PUMP.trash_tx.send(target.into()).ok();
	}

	pub fn push_delete<U>(target: U)
	where
		U: Into<UrlBuf>,
	{
		PUMP.delete_tx.send(target.into()).ok();
	}

	pub fn push_download<U>(target: U)
	where
		U: Into<UrlBuf>,
	{
		PUMP.download_tx.send(target.into()).ok();
	}

	pub(super) fn serve() {
		let (move_tx, move_rx) = mpsc::unbounded_channel();
		let (duplicate_tx, duplicate_rx) = mpsc::unbounded_channel();
		let (trash_tx, trash_rx) = mpsc::unbounded_channel();
		let (delete_tx, delete_rx) = mpsc::unbounded_channel();
		let (download_tx, download_rx) = mpsc::unbounded_channel();
		let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

		PUMP.init(Self { duplicate_tx, move_tx, trash_tx, delete_tx, download_tx, shutdown_tx });

		tokio::spawn(async move {
			let duplicate_rx =
				UnboundedReceiverStream::new(duplicate_rx).chunks_timeout(1000, Duration::from_millis(500));
			let move_rx =
				UnboundedReceiverStream::new(move_rx).chunks_timeout(1000, Duration::from_millis(500));
			let trash_rx =
				UnboundedReceiverStream::new(trash_rx).chunks_timeout(1000, Duration::from_millis(500));
			let delete_rx =
				UnboundedReceiverStream::new(delete_rx).chunks_timeout(1000, Duration::from_millis(500));
			let download_rx =
				UnboundedReceiverStream::new(download_rx).chunks_timeout(1000, Duration::from_millis(500));

			pin!(duplicate_rx);
			pin!(move_rx);
			pin!(trash_rx);
			pin!(delete_rx);
			pin!(download_rx);

			loop {
				select! {
					Some(items) = duplicate_rx.next() => log_if_err!(Pubsub::pub_after_duplicate(items)),
					Some(items) = move_rx.next() => log_if_err!(Pubsub::pub_after_move(items)),
					Some(urls) = trash_rx.next() => log_if_err!(Pubsub::pub_after_trash(urls)),
					Some(urls) = delete_rx.next() => log_if_err!(Pubsub::pub_after_delete(urls)),
					Some(urls) = download_rx.next() => log_if_err!(Pubsub::pub_after_download(urls)),
					_ = shutdown_rx.recv() => {
						shutdown_rx.close();
						break;
					}
				}
			}
		});
	}

	pub(super) async fn shutdown() {
		PUMP.shutdown_tx.send(()).ok();
		PUMP.shutdown_tx.closed().await;
	}
}
