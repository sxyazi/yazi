#[derive(Debug)]
pub enum PreworkOut {
	Fetch(PreworkOutFetch),
	Load(PreworkOutLoad),
	Size(PreworkOutSize),
}

#[derive(Debug)]
pub struct PreworkOutFetch;

#[derive(Debug)]
pub struct PreworkOutLoad;

#[derive(Debug)]
pub struct PreworkOutSize;
