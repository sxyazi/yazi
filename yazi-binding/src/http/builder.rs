use std::{io, path::PathBuf};

use mlua::{AnyUserData, BorrowedBytes, ExternalResult, UserData, UserDataMethods};
use reqwest::{Method, header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue}};
use yazi_shared::path::PathBufDyn;

use super::{HttpPart, HttpRequest, HttpSession, HttpSpec, send};

pub struct HttpBuilder {
	url:     String,
	method:  Method,
	headers: HeaderMap,
	socket:  PathBuf,
	spec:    HttpSpec,
}

impl HttpBuilder {
	pub fn new(method: &[u8], url: String) -> io::Result<Self> {
		Ok(Self {
			url,
			method: Method::from_bytes(method).map_err(io::Error::other)?,
			headers: HeaderMap::new(),
			socket: PathBuf::new(),
			spec: Default::default(),
		})
	}

	fn start(self) -> HttpSession {
		let Self { url, method, mut headers, socket, mut spec } = self;

		if let Some(content_type) = spec.content_type() {
			headers.entry(CONTENT_TYPE).or_insert(content_type);
		}

		let tx = spec.take_tx();
		let request = HttpRequest { url, method, headers, spec, socket };

		let handle = tokio::spawn(async move { send(request).await });
		HttpSession { tx, handle: Some(handle) }
	}
}

impl UserData for HttpBuilder {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function(
			"header",
			|_, (ud, name, value): (AnyUserData, BorrowedBytes, BorrowedBytes)| {
				let name = HeaderName::from_bytes(&name).map_err(mlua::Error::external)?;
				let value = HeaderValue::from_bytes(&value).map_err(mlua::Error::external)?;
				ud.borrow_mut::<Self>()?.headers.insert(name, value);
				Ok(ud)
			},
		);
		methods.add_function("socket", |_, (ud, socket): (AnyUserData, PathBufDyn)| {
			ud.borrow_mut::<Self>()?.socket = socket.into_os()?;
			Ok(ud)
		});
		methods.add_function("body", |_, (ud, bytes): (AnyUserData, Vec<u8>)| {
			ud.borrow_mut::<Self>()?.spec.bytes(bytes);
			Ok(ud)
		});
		methods.add_function("raw", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.spec.raw();
			Ok(ud)
		});
		methods.add_function("field", |_, (ud, name, value): (AnyUserData, String, String)| {
			ud.borrow_mut::<Self>()?.spec.field(name, value);
			Ok(ud)
		});
		methods.add_function("part", |_, (ud, part): (AnyUserData, HttpPart)| {
			ud.borrow_mut::<Self>()?.spec.part(part).into_lua_err()?;
			Ok(ud)
		});
		methods.add_method_once("start", |_, me, ()| Ok(me.start()));
	}
}
