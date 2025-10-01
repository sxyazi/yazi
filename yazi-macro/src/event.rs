#[macro_export]
macro_rules! emit {
	(Quit($opt:expr)) => {
		yazi_shared::event::Event::Quit($opt).emit();
	};
	(Call($cmd:expr)) => {
		yazi_shared::event::Event::Call(yazi_shared::event::CmdCow::from($cmd)).emit();
	};
	(Seq($cmds:expr)) => {
		yazi_shared::event::Event::Seq($cmds).emit();
	};
	($event:ident) => {
		yazi_shared::event::Event::$event.emit();
	};
}

#[macro_export]
macro_rules! relay {
	($layer:ident : $name:ident) => {
		yazi_shared::event::Cmd::new_relay(concat!(stringify!($layer), ":", stringify!($name)))
	};
	($layer:ident : $name:ident, $args:expr) => {
		yazi_shared::event::Cmd::new_relay_args(
			concat!(stringify!($layer), ":", stringify!($name)),
			$args,
		)
	};
}

#[macro_export]
macro_rules! succ {
	($data:expr) => {
		return Ok(yazi_shared::data::Data::from($data))
	};
	() => {
		return Ok(yazi_shared::data::Data::Nil)
	};
}
