use ratatui::{text::Text, widgets::{Paragraph, Wrap}};

pub fn line_count<'a, T, W, I>(text: T, width: u16, indent: I, wrap: W) -> usize
where
	T: Into<Text<'a>>,
	I: AsRef<str>,
	W: Into<Option<Wrap>>,
{
	line_count_impl(text.into(), width, indent.as_ref(), wrap.into())
}

fn line_count_impl(mut text: Text<'_>, mut width: u16, indent: &str, wrap: Option<Wrap>) -> usize {
	width = width.max(1);

	let Some(wrap) = wrap else {
		return Paragraph::new(text).line_count(width);
	};

	if indent.len() == 1 {
		return Paragraph::new(text).wrap(wrap).line_count(width);
	}

	let extra = indent.len().saturating_sub(1);
	for line in &mut text.lines {
		for span in &mut line.spans {
			let mut out = None::<String>;
			let mut start = 0;
			for (idx, b) in span.content.bytes().enumerate() {
				if b != b'\t' {
					continue;
				}

				let out = out.get_or_insert_with(|| String::with_capacity(span.content.len() + extra));
				if start < idx {
					out.push_str(unsafe { span.content.get_unchecked(start..idx) });
				}

				out.push_str(indent);
				start = idx + 1;
			}

			if let Some(mut out) = out {
				if start < span.content.len() {
					out.push_str(unsafe { span.content.get_unchecked(start..) });
				}
				span.content = out.into();
			}
		}
	}

	Paragraph::new(text).wrap(wrap).line_count(width)
}
