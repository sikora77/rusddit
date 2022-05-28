use tui::{
	backend::Backend,
	layout::{Alignment, Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	text::Spans,
	widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
	Frame,
};

pub fn draw_first_tab<B>(
	f: &mut Frame<B>,
	app: &mut crate::app::App,
	v: &Vec<serde_json::Value>,
	tabs: Tabs,
) where
	B: Backend,
{
	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
		.split(size);
	f.render_widget(tabs, chunks[0]);
	let items = &app.items.items;
	let items: Vec<ListItem> = items
		.iter()
		.map(|i| {
			let p = i.0.clone();
			let mut lines = vec![Spans::from(p)];
			let mut content = match v[i.1 as usize]["data"]["selftext"].as_str() {
				Some(x) => x,
				None => "Not a text post",
			};
			if content == "" {
				content = "Not a text post";
			}
			lines.push(Spans::from(content));
			ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
		})
		.collect();
	let items = List::new(items)
		.block(Block::default().borders(Borders::ALL).title("Posts"))
		.highlight_style(
			Style::default()
				.bg(Color::Blue)
				.fg(Color::Black)
				.add_modifier(Modifier::BOLD),
		)
		.highlight_symbol(">> ");
	f.render_stateful_widget(items, chunks[1], &mut app.items.state);
}
pub fn draw_second_tab<B>(
	f: &mut Frame<B>,
	app: &mut crate::app::App,
	v: &Vec<serde_json::Value>,
	tabs: Tabs,
) where
	B: Backend,
{
	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints(
			[
				Constraint::Length(3),
				Constraint::Percentage(20),
				Constraint::Percentage(50),
				Constraint::Min(0),
			]
			.as_ref(),
		)
		.split(size);
	f.render_widget(tabs, chunks[0]);
	let index = match app.items.state.selected() {
		Some(x) => x,
		None => 0,
	};
	let title = match v[index]["data"]["title"].as_str() {
		Some(x) => x,
		None => "Post",
	};
	let subreddit = match v[index]["data"]["subreddit_name_prefixed"].as_str() {
		Some(x) => x,
		None => "Post",
	};
	let widget = Block::default().title(subreddit).borders(Borders::ALL);
	let title_paragraph = Paragraph::new(title)
		.style(Style::default().bg(Color::Black).fg(Color::White))
		.block(widget)
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true });
	f.render_widget(title_paragraph, chunks[1]);

	let widget2 = Block::default()
		.title("Post")
		.borders(Borders::ALL)
		.border_style(Style::default().fg(match app.current_focus {
			0 => Color::Red,
			_ => Color::White,
		}));
	let data = v[index]["data"]["selftext"].clone();
	let text;
	let c = match data.as_str() {
		Some(x) => x,
		None => "Not a text post",
	};
	text = vec![Spans::from(c)];
	let paragraph = Paragraph::new(text.clone())
		.style(Style::default().bg(Color::Black).fg(Color::White))
		.block(widget2)
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true })
		.scroll((app.post_scroll, 0));
	f.render_widget(paragraph, chunks[2]);

	let widget3 = Block::default()
		.title("Comments")
		.borders(Borders::ALL)
		.border_style(Style::default().fg(match app.current_focus {
			1 => Color::Red,
			_ => Color::White,
		}));
	let comment_count = app.comments[1]["data"]["children"]
		.as_array()
		.unwrap()
		.len();
	let mut comments = "".to_string();
	for i in 0..comment_count {
		let comment_text = match app.comments[1]["data"]["children"][i]["data"]["body"].as_str() {
			Some(x) => x,
			None => "",
		};
		let comment_author = app.comments[1]["data"]["children"][i]["data"]["author"]
			.as_str()
			.unwrap_or_default();
		let text = format!(
			"{}{}{}{}{}",
			"u/", &comment_author, " | ", comment_text, "\n"
		);
		comments = format!("{}{}", comments, text);
	}
	let comment_paragraph = Paragraph::new(comments)
		.style(Style::default().bg(Color::Black).fg(Color::White))
		.block(widget3)
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true })
		.scroll((app.comment_scroll, 0));
	f.render_widget(comment_paragraph, chunks[3]);
}

pub fn draw_third_tab<B>(f: &mut Frame<B>, app: &mut crate::app::App, tabs: Tabs)
where
	B: Backend,
{
	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
		.split(size);
	f.render_widget(tabs, chunks[0]);
	let second_chunk = Layout::default()
		.direction(Direction::Vertical)
		.vertical_margin(13)
		.horizontal_margin(25)
		.constraints([Constraint::Min(0)].as_ref())
		.split(chunks[1]);
	let widget = Block::default()
		.title("Search")
		.borders(Borders::ALL)
		.title_alignment(Alignment::Center);
	let title_paragraph = Paragraph::new(app.input.clone())
		.style(Style::default().bg(Color::White).fg(Color::Black))
		.block(widget)
		.alignment(Alignment::Center)
		.wrap(Wrap { trim: true });
	f.render_widget(title_paragraph, second_chunk[0]);
}
