use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, fs, io, path::Path};
use tui::{
	backend::{Backend, CrosstermBackend},
	layout::{Alignment, Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
	Frame, Terminal,
};

struct StatefulList<T> {
	state: ListState,
	items: Vec<T>,
}

static mut LAST_POST_ID: String = String::new();

impl<T> StatefulList<T> {
	fn with_items(items: Vec<T>) -> StatefulList<T> {
		let mut sl = StatefulList {
			state: ListState::default(),
			items,
		};
		sl.state.select(Some(0));
		return sl;
	}

	fn next(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	fn previous(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}
}

struct App<'a> {
	pub titles: Vec<&'a str>,
	items: StatefulList<(String, usize)>,
	pub index: usize,
	post_scroll: u16,
	comment_scroll: u16,
	input: String,
	comments: serde_json::Value,
	sort_by: String,
	comments_sort_by: String,
	current_focus: usize,
}

impl<'a> App<'a> {
	fn new() -> App<'a> {
		App {
			titles: vec!["Home", "Post", "Search"],
			current_focus: 0,
			index: 0,
			items: StatefulList::with_items(vec![("Item0".to_string(), 1)]),
			post_scroll: 0,
			input: "".to_owned(),
			comments: serde_json::from_str("{\"foo\":\"bar\"}").unwrap(),
			sort_by: "hot".to_string(),
			comments_sort_by: "best".to_string(),
			comment_scroll: 0,
		}
	}
	fn post_scroll_up(&mut self, ammount: u16) {
		if self.post_scroll >= 1 {
			self.post_scroll -= ammount;
		}
	}
	fn post_scroll_down(&mut self, ammount: u16) {
		self.post_scroll += ammount;
	}
	fn comment_scroll_up(&mut self, ammount: u16) {
		if self.comment_scroll >= 1 {
			self.comment_scroll -= ammount;
		}
	}
	fn comment_scroll_down(&mut self, ammount: u16) {
		self.comment_scroll += ammount;
	}

	pub fn next(&mut self) {
		self.index = (self.index + 1) % self.titles.len();
	}

	pub fn previous(&mut self) {
		if self.index > 0 {
			self.index -= 1;
		} else {
			self.index = self.titles.len() - 1;
		}
	}
	fn append_input(&mut self, user_char: char) {
		self.input.insert(self.input.len(), user_char);
	}
	fn delete_from_input(&mut self) {
		self.input.pop();
	}
	fn update_comments(&mut self, val: Vec<serde_json::Value>, index: usize) {
		let post_id = &(match val[index]["data"]["name"].as_str() {
			Some(x) => x,
			None => panic!("this shouldn't happen"),
		})[3..];
		let url = format!(
			"{}{}{}{}{}",
			"https://reddit.com/comments/", post_id, "/", self.comments_sort_by, ".json"
		);
		// println!("{}", url);
		let body = match reqwest::blocking::get(&url) {
			Ok(x) => x,
			Err(err) => {
				panic!("called `Result::unwrap()` on an `Err` value: {:?}", err)
			}
		};
		let body = match body.text() {
			Ok(x) => x,
			Err(err) => {
				panic!("called `Result::unwrap()` on an `Err` value: {:?}", err)
			}
		};
		self.comments = serde_json::from_str(&body).unwrap();
	}
	fn change_focus(&mut self) {
		self.current_focus = (self.current_focus + 1) % 2;
	}
}
fn filter_out_text_posts(v: serde_json::Value) -> Vec<serde_json::Value> {
	let ammount = match v["data"]["dist"].as_i64() {
		Some(x) => x,
		None => 0,
	};
	unsafe {
		LAST_POST_ID = v["data"]["children"][ammount as usize - 1]["data"]["name"]
			.as_str()
			.expect("msg")
			.clone()
			.to_string();
	}

	let p = match v["data"]["children"].as_array() {
		Some(x) => x.clone(),
		None => panic!(),
	};
	let s = p
		.iter()
		.cloned()
		.filter(|x| x.clone()["data"]["selftext"] != "");
	//.filter(|x| true);
	let m: Vec<serde_json::Value> = s.collect();
	return m;
}

fn get_posts(input: String, before: bool, sort_by: String) -> Vec<serde_json::Value> {
	let slash = match input.as_str() {
		"" => "",
		_ => "/",
	};
	let mut url = format!(
		"{}{}{}{}{}",
		"https://reddit.com/", input, slash, sort_by, ".json?limit=100"
	);
	unsafe {
		if before {
			url = format!("{}{}{}", url, "&after=", LAST_POST_ID)
		}
	}
	let reddit_cookie = match std::fs::read_to_string(Path::join(
		home::home_dir().expect("what").as_path(),
		Path::new(".config/rusddit/cookie.txt"),
	)) {
		Ok(x) => x,
		Err(x) => {
			println!("Couldn't find cookie file. Starting an anonymous session");
			"".to_string()
		}
	};
	let cookie = &format!(
		"{}{}{}",
		"reddit_session=", reddit_cookie, "; Domain=reddit.com"
	);
	let cookie_url = "https://reddit.com".parse::<reqwest::Url>().unwrap();

	let jar = reqwest::cookie::Jar::default();
	jar.add_cookie_str(cookie, &cookie_url);
	let cookie_store = std::sync::Arc::new(jar);

	let client_builder = reqwest::blocking::Client::builder()
		.cookie_store(true)
		.cookie_provider(cookie_store);
	let client = client_builder.build().expect("Fuuuck");
	let req = client.get(&url).build().expect("fuck");
	let client_body = client.execute(req).expect("client request failed");
	let body = match client_body.text() {
		Ok(x) => x,
		Err(err) => {
			panic!("called `Result::unwrap()` on an `Err` value: {:?}", err)
		}
	};
	return filter_out_text_posts(match serde_json::from_str(&body) {
		Ok(x) => x,
		Err(x) => panic!("{:?}", x),
	})
	.to_owned();
}

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = std::env::args().collect();
	for i in 0..args.len() {
		let path = Path::join(
			home::home_dir().expect("what").as_path(),
			Path::new(".config/rusddit/cookie.txt"),
		);
		if args[i] == "-c" || args[i] == "--cookie" {
			if Path::exists(path.as_path()) == false {
				match fs::create_dir_all(path.as_path()) {
					Ok(x) => {
						let res = fs::write(path.as_path(), args[i + 1].clone());
					}
					Err(x) => {
						println!("{}", "Couldn't create cookie file")
					}
				}
			}
		}
	}
	let m = get_posts("".to_owned(), false, "hot".to_string());
	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	let app = App::new();
	// app.update_comments(m.clone(), 0);
	let res = run_app(&mut terminal, app, m);
	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("{:?}", err)
	}

	Ok(())
}

fn run_app<B: Backend>(
	terminal: &mut Terminal<B>,
	mut app: App,
	mut v: Vec<serde_json::Value>,
) -> io::Result<()> {
	loop {
		terminal.draw(|f| ui(f, &mut app, &mut v))?;
		if let Event::Key(key) = event::read()? {
			if app.index == 0 {
				match key.code {
					KeyCode::Esc => return Ok(()),
					KeyCode::Char('2') => app.next(),
					KeyCode::Char('1') => app.previous(),
					KeyCode::Left => app.previous(),
					KeyCode::Up => app.items.previous(),
					KeyCode::Char('h') => {
						app.sort_by = "hot".to_string();
						v = get_posts(app.input.clone(), false, app.sort_by.clone());
					}
					KeyCode::Char('b') => {
						app.sort_by = "best".to_string();
						v = get_posts(app.input.clone(), false, app.sort_by.clone());
					}
					KeyCode::Char('c') => {
						app.sort_by = "controversial".to_string();
						v = get_posts(app.input.clone(), false, app.sort_by.clone());
					}
					KeyCode::Right => {
						app.update_comments(v.clone(), app.items.state.selected().unwrap());
						app.next();
					}
					KeyCode::Down => {
						app.items.next();
						if app.items.state.selected() == Some(app.items.items.len()) {
							v = get_posts(app.input.clone(), true, app.sort_by.clone());
							app.items.state.select(Some(0));
						}
					}
					_ => {}
				}
			} else if app.index == 1 {
				match key.code {
					KeyCode::Esc => return Ok(()),
					KeyCode::Tab => {
						app.change_focus();
					}
					KeyCode::Char('2') => app.next(),
					KeyCode::Char('1') => app.previous(),
					KeyCode::Left => app.previous(),
					KeyCode::Char('j') => {
						if app.current_focus == 0 {
							app.post_scroll_up(1);
						} else {
							app.comment_scroll_up(1);
						}
					}
					KeyCode::Right => app.next(),
					KeyCode::Char('k') => {
						if app.current_focus == 0 {
							app.post_scroll_down(1);
						} else {
							app.comment_scroll_down(1);
						}
					}
					KeyCode::Char('h') => {
						app.comments_sort_by = "hot".to_string();
						app.update_comments(v.to_owned(), app.items.state.selected().unwrap());
					}
					KeyCode::Char('b') => {
						app.comments_sort_by = "best".to_string();
						app.update_comments(v.to_owned(), app.items.state.selected().unwrap());
					}
					KeyCode::Char('c') => {
						app.comments_sort_by = "controversial".to_string();
						app.update_comments(v.to_owned(), app.items.state.selected().unwrap());
					}
					KeyCode::Down => {
						app.items.next();
						if app.items.state.selected() == Some(app.items.items.len()) {
							v = get_posts(app.input.clone(), true, app.sort_by.clone());
							app.items.state.select(Some(0));
						}
						app.update_comments(v.to_owned(), app.items.state.selected().unwrap());
					}
					KeyCode::Up => {
						app.items.previous();
						app.update_comments(v.to_owned(), app.items.state.selected().unwrap());
					}
					_ => {}
				}
			} else if app.index == 2 {
				match key.code {
					KeyCode::Esc => return Ok(()),
					KeyCode::Char(c) => {
						app.append_input(c);
					}
					KeyCode::Backspace => {
						app.delete_from_input();
					}
					KeyCode::Enter => {
						v = get_posts(app.input.clone(), false, app.sort_by.clone());
						app.items.state.select(Some(0));
						// app.index = 0;
					}
					KeyCode::Left => {
						app.previous();
						app.update_comments(v.clone(), app.items.state.selected().unwrap());
					}
					KeyCode::Right => app.next(),
					_ => {}
				}
			}
		}
	}
}

fn ui<B: Backend>(f: &mut Frame<B>, mut app: &mut App, v: &Vec<serde_json::Value>) {
	let mut posts: Vec<(String, usize)> = Vec::new();
	for i in 0..v.len() {
		let formatted = format!(
			"{}{}",
			v[i as usize]["data"]["subreddit_name_prefixed"], v[i as usize]["data"]["title"]
		);
		posts.push((formatted, i as usize));
	}

	app.items.items = posts.clone();
	let size = f.size();

	let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
	f.render_widget(block, size);
	let titles = app
		.titles
		.iter()
		.map(|t| {
			let (first, rest) = t.split_at(1);
			Spans::from(vec![
				Span::styled(first, Style::default().fg(Color::Yellow)),
				Span::styled(rest, Style::default().fg(Color::Green)),
			])
		})
		.collect();
	let tabs = Tabs::new(titles)
		.block(Block::default().borders(Borders::ALL).title("Tabs"))
		.select(app.index)
		.style(Style::default().fg(Color::Cyan))
		.highlight_style(
			Style::default()
				.add_modifier(Modifier::BOLD)
				.bg(Color::White)
				.fg(Color::Black),
		);

	match app.index {
		0 => draw_first_tab(f, &mut app, v, tabs),
		1 => draw_second_tab(f, &mut app, v, tabs),
		2 => draw_third_tab(f, &mut app, tabs),
		_ => unreachable!(),
	};
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, v: &Vec<serde_json::Value>, tabs: Tabs)
where
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
fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, v: &Vec<serde_json::Value>, tabs: Tabs)
where
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

fn draw_third_tab<B>(f: &mut Frame<B>, app: &mut App, tabs: Tabs)
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
