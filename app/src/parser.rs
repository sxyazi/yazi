use config::theme::Color;
use ratatui::{style::{Modifier, Style}, text::{Line, Span}, widgets::Paragraph};

pub struct Parser;

impl Parser {
	pub fn span(s: &str) -> Span<'static> {
		let Some((args, content)) = s.split_once(';') else {
			return Span::raw(s.to_string());
		};

		let args = args.split(',').collect::<Vec<_>>();
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

	pub fn line(s: &str) -> Line<'static> {
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

	pub fn paragraph(s: &str) -> Paragraph {
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

	pub fn layout(s: &str) -> Paragraph {
		let mut last = '\0';
		let mut lines: Vec<String> = vec![String::new()];

		for c in s.chars() {
			if c == '\0' && last == '\\' {
				let last = lines.last_mut().unwrap();
				last.pop();
				last.push(c);
			} else if c == '\0' {
				lines.push(String::new());
			} else {
				lines.last_mut().unwrap().push(c);
			}
			last = c;
		}
		Paragraph::new(lines.into_iter().map(|s| Self::line(&s)).collect::<Vec<_>>())
	}
}
