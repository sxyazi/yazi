use std::fmt::{self, Display};

/// XTVERSION request (secondary DA)
pub struct RequestXtVersion;

impl Display for RequestXtVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[>q") }
}

/// Request character cell pixel size
pub struct RequestCellPixelSize;

impl Display for RequestCellPixelSize {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[16t") }
}

/// Request background color via OSC 11
pub struct RequestBgColor;

impl Display for RequestBgColor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b]11;?\x07") }
}

/// Request device attributes DA1
pub struct RequestDA1;

impl Display for RequestDA1 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[0c") }
}

/// Query Kitty graphics protocol capabilities
pub struct KittyGraphicsQuery;

impl Display for KittyGraphicsQuery {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\")
	}
}

/// Request cursor style via DECRQSS (DECSCUSR)
pub struct RequestCursorStyle;

impl Display for RequestCursorStyle {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1bP$q q\x1b\\") }
}

/// Request cursor blink status via DECRQM (DECSET 12)
pub struct RequestCursorBlink;

impl Display for RequestCursorBlink {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[?12$p") }
}

/// Device Status Report (DSR)
pub struct RequestDeviceStatus;

impl Display for RequestDeviceStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[5n") }
}
