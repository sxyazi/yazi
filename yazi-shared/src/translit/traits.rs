use core::str;
use std::borrow::Cow;

pub trait Transliterator {
	fn transliterate(&self) -> Cow<str>;
}

impl Transliterator for &[u8] {
	fn transliterate(&self) -> Cow<str> {
		// Fast path to skip over ASCII chars at the beginning of the string
		let ascii_len = self.iter().take_while(|&&c| c < 0x7f).count();
		if ascii_len >= self.len() {
			return Cow::Borrowed(unsafe { str::from_utf8_unchecked(self) });
		}

		let (ascii, rest) = self.split_at(ascii_len);

		// Reserve a bit more space to avoid reallocations on longer transliterations
		// but instead of `+ 16` uses `| 15` to stay in the smallest allocation bucket
		// for short strings
		let mut out = String::new();
		out.try_reserve_exact(self.len() | 15).unwrap_or_else(|_| panic!());
		out.push_str(unsafe { str::from_utf8_unchecked(ascii) });

		for c in String::from_utf8_lossy(rest).chars() {
			if let Some(s) = super::lookup(c) {
				out.push_str(s);
			} else {
				out.push(c);
			}
		}
		Cow::Owned(out)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_transliterate() {
		assert_eq!("Æcœ".as_bytes().transliterate(), "AEcoe");
		assert_eq!(
			"ěřůøĉĝĥĵŝŭèùÿėįųāēīūļķņģőűëïąćęłńśźżõșțčďĺľňŕšťýžéíñóúüåäöçîşûğăâđêôơưáàãảạ"
				.as_bytes()
				.transliterate(),
			"eruocghjsueuyeiuaeiulkngoueiacelnszzostcdllnrstyzeinouuaaocisugaadeoouaaaaa",
		);
		assert_eq!(
			"áạàảãăắặằẳẵâấậầẩẫéẹèẻẽêếệềểễiíịìỉĩoóọòỏõôốộồổỗơớợờởỡúụùủũưứựừửữyýỵỳỷỹđ"
				.as_bytes()
				.transliterate(),
			"aaaaaaaaaaaaaaaaaeeeeeeeeeeeiiiiiioooooooooooooooooouuuuuuuuuuuyyyyyyd",
		);
		assert_ne!(
			"ěřůøĉĝĥĵŝŭèùÿėįųāēīūļķņģőűëïąćęłńśźżõșțčďĺľňŕšťýžéíñóúüåäöçîşûğăâđêôơưáàãảạﬁﬂ"
				.as_bytes()
				.transliterate(),
			"ěřůøĉĝĥĵŝŭèùÿėįųāēīūļķņģőűëïąćęłńśźżõșțčďĺľňŕšťýžéíñóúüåäöçîşûğăâđêôơưáàãảạfifl"
		);
		assert_eq!(
			"THEQUICKBROWNFOXJUMPEDOVERTHELAZYDOGthequickbrownfoxjumpedoverthelazydog"
				.as_bytes()
				.transliterate(),
			"THEQUICKBROWNFOXJUMPEDOVERTHELAZYDOGthequickbrownfoxjumpedoverthelazydog"
		);
	}
}
