use anyhow::Result;
use config::theme::Color;
use ratatui::{prelude::{Alignment, Buffer, Rect}, style::{Modifier, Style}, text::{Line, Span}, widgets::{Paragraph, Widget}};

pub struct Parser;

impl Parser {
	fn span(s: &str) -> Span<'static> {
		let Some((args, content)) = s.split_once(';') else {
			return Span::raw(s.to_string());
		};

		let args: Vec<_> = args.split(',').collect();
		if args.len() != 4 {
			return Span::raw(s.to_string());
		}

		let mut style = Style::new();
		if let Ok(fg) = Color::try_from(args[0]) {
			style = style.fg(fg.into());
		}
		if let Ok(bg) = Color::try_from(args[1]) {
			style = style.bg(bg.into());
		}
		if let Ok(uc) = Color::try_from(args[2]) {
			style = style.underline_color(uc.into());
		}
		if let Ok(modifier) = args[3].parse::<u16>() {
			style = style.add_modifier(Modifier::from_bits_truncate(modifier));
		}
		Span::styled(content.to_string(), style)
	}

	fn line(s: &str) -> Line<'static> {
		let mut last = '\0';
		let mut spans: Vec<String> = vec![String::new()];

		for c in s.chars() {
			if c == '\n' && last == '\\' {
				let last = spans.last_mut().unwrap();
				last.pop();
				last.push(c);
			} else if c == '\n' {
				spans.push(String::new());
			} else {
				spans.last_mut().unwrap().push(c);
			}
			last = c;
		}
		Line::from(spans.into_iter().map(|s| Self::span(&s)).collect::<Vec<_>>())
	}

	fn paragraph(s: &str) -> Paragraph {
		let mut last = '\0';
		let mut lines: Vec<String> = vec![String::new()];

		for c in s.chars() {
			if c == '\r' && last == '\\' {
				let last = lines.last_mut().unwrap();
				last.pop();
				last.push(c);
			} else if c == '\r' {
				lines.push(String::new());
			} else {
				lines.last_mut().unwrap().push(c);
			}
			last = c;
		}
		Paragraph::new(lines.into_iter().map(|s| Self::line(&s)).collect::<Vec<_>>())
	}

	fn area(args: &[&str]) -> Result<Rect> {
		Ok(Rect {
			x:      args[0].parse()?,
			y:      args[1].parse()?,
			width:  args[2].parse()?,
			height: args[3].parse()?,
		})
	}

	pub fn render(s: &str, buf: &mut Buffer) {
		let Some(s) = s.strip_prefix('R') else {
			return;
		};

		let mut last = '\0';
		let mut paragraphs: Vec<String> = vec![String::new()];

		for c in s.chars() {
			if c == '\0' && last == '\\' {
				let last = paragraphs.last_mut().unwrap();
				last.pop();
				last.push(c);
			} else if c == '\0' {
				paragraphs.push(String::new());
			} else {
				paragraphs.last_mut().unwrap().push(c);
			}
			last = c;
		}

		for paragraph in paragraphs {
			let Some((args, content)) = paragraph.split_once(';') else {
				continue;
			};

			let args: Vec<_> = args.split(',').collect();
			if args.len() != 5 {
				continue;
			}

			let Ok(area) = Self::area(&args) else {
				continue;
			};

			let mut paragraph = Self::paragraph(content);
			if let Ok(align) = args[4].parse::<u8>() {
				paragraph = paragraph.alignment(match align {
					1 => Alignment::Center,
					2 => Alignment::Right,
					_ => Alignment::Left,
				});
			}

			paragraph.render(area, buf);
		}
	}
}
