use mlua::SerializeOptions;

pub const SER_OPT: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);
