use std::time::Duration;

use tokio::{pin, select, sync::mpsc};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use yazi_macro::err;
use yazi_shared::{RoCell, url::UrlBuf};

use crate::{Pubsub, ember::{BodyDuplicateItem, BodyMoveItem}};

static DUPLICATE_TX: RoCell<mpsc::UnboundedSender<BodyDuplicateItem>> = RoCell::new();
static MOVE_TX: RoCell<mpsc::UnboundedSender<BodyMoveItem>> = RoCell::new();
static TRASH_TX: RoCell<mpsc::UnboundedSender<UrlBuf>> = RoCell::new();
static DELETE_TX: RoCell<mpsc::UnboundedSender<UrlBuf>> = RoCell::new();
static SHUTDOWN_TX: RoCell<mpsc::UnboundedSender<()>> = RoCell::new();

pub struct Pump;

impl Pump {
	pub fn push_duplicate<U>(from: U, to: U)
	where
		U: Into<UrlBuf>,
	{
		DUPLICATE_TX.send(BodyDuplicateItem { from: from.into(), to: to.into() }).ok();
	}

	pub fn push_move<U>(from: U, to: U)
	where
		U: Into<UrlBuf>,
	{
		MOVE_TX.send(BodyMoveItem { from: from.into(), to: to.into() }).ok();
	}

	pub fn push_trash<U>(target: U)
	where
		U: Into<UrlBuf>,
	{
		TRASH_TX.send(target.into()).ok();
	}

	pub fn push_delete<U>(target: U)
	where
		U: Into<UrlBuf>,
	{
		DELETE_TX.send(target.into()).ok();
	}

	pub(super) fn serve() {
		let (move_tx, move_rx) = mpsc::unbounded_channel();
		let (duplicate_tx, duplicate_rx) = mpsc::unbounded_channel();
		let (trash_tx, trash_rx) = mpsc::unbounded_channel();
		let (delete_tx, delete_rx) = mpsc::unbounded_channel();
		let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

		DUPLICATE_TX.init(duplicate_tx);
		MOVE_TX.init(move_tx);
		TRASH_TX.init(trash_tx);
		DELETE_TX.init(delete_tx);
		SHUTDOWN_TX.init(shutdown_tx);

		tokio::spawn(async move {
			let duplicate_rx =
				UnboundedReceiverStream::new(duplicate_rx).chunks_timeout(1000, Duration::from_millis(500));
			let move_rx =
				UnboundedReceiverStream::new(move_rx).chunks_timeout(1000, Duration::from_millis(500));
			let trash_rx =
				UnboundedReceiverStream::new(trash_rx).chunks_timeout(1000, Duration::from_millis(500));
			let delete_rx =
				UnboundedReceiverStream::new(delete_rx).chunks_timeout(1000, Duration::from_millis(500));

			pin!(duplicate_rx);
			pin!(move_rx);
			pin!(trash_rx);
			pin!(delete_rx);

			loop {
				select! {
					Some(items) = duplicate_rx.next() => err!(Pubsub::pub_after_duplicate(items)),
					Some(items) = move_rx.next() => err!(Pubsub::pub_after_move(items)),
					Some(urls) = trash_rx.next() => err!(Pubsub::pub_after_trash(urls)),
					Some(urls) = delete_rx.next() => err!(Pubsub::pub_after_delete(urls)),
					_ = shutdown_rx.recv() => {
						shutdown_rx.close();
						break;
					}
				}
			}
		});
	}

	pub(super) async fn shutdown() {
		SHUTDOWN_TX.send(()).ok();
		SHUTDOWN_TX.closed().await;
	}
}
