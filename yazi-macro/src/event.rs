#[macro_export]
macro_rules! emit {
	(Call($action:expr)) => {
		yazi_shared::event::Event::Call(yazi_shared::event::ActionCow::from($action)).emit();
	};
	(Seq($actions:expr)) => {
		yazi_shared::event::Event::Seq($actions).emit();
	};
	($event:ident) => {
		yazi_shared::event::Event::$event.emit();
	};
}

#[macro_export]
macro_rules! relay {
	($layer:ident : $name:ident) => {
		yazi_shared::event::Action::new_relay(concat!(stringify!($layer), ":", stringify!($name)))
	};
	($layer:ident : $name:ident, $args:expr) => {
		yazi_shared::event::Action::new_relay_args(
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
