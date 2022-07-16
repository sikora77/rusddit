use crossterm::event::{self, Event, KeyCode};
use std::io;
use std::path::Path;
use tui::{backend::Backend, widgets::ListState, Terminal};

use crate::ui;

use crate::utils;

pub struct StatefulList<T> {
	pub state: ListState,
	pub items: Vec<T>,
}

impl<T> StatefulList<T> {
	fn with_items(items: Vec<T>) -> StatefulList<T> {
		let mut sl = StatefulList {
			state: ListState::default(),
			items,
		};
		sl.state.select(Some(0));
		return sl;
	}

	pub fn next(&mut self) {
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

	pub fn previous(&mut self) {
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

pub struct App<'a> {
	pub titles: Vec<&'a str>,
	pub items: StatefulList<(String, usize)>,
	pub index: usize,
	pub post_scroll: u16,
	pub comment_scroll: u16,
	pub input: String,
	pub comments: serde_json::Value,
	pub sort_by: String,
	pub comments_sort_by: String,
	pub current_focus: usize,
	pub cookie: String,
}

impl<'a> App<'a> {
	pub fn new(cookie: String) -> App<'a> {
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
			cookie: cookie,
		}
	}
	pub fn post_scroll_up(&mut self, ammount: u16) {
		if self.post_scroll >= 1 {
			self.post_scroll -= ammount;
		}
	}
	pub fn post_scroll_down(&mut self, ammount: u16) {
		self.post_scroll += ammount;
	}
	pub fn comment_scroll_up(&mut self, ammount: u16) {
		if self.comment_scroll >= 1 {
			self.comment_scroll -= ammount;
		}
	}
	pub fn comment_scroll_down(&mut self, ammount: u16) {
		self.comment_scroll += ammount;
	}

	pub fn next(&mut self) {
		self.comment_scroll = 0;
		self.post_scroll = 0;
		self.index = (self.index + 1) % self.titles.len();
	}

	pub fn previous(&mut self) {
		self.comment_scroll = 0;
		self.post_scroll = 0;
		if self.index > 0 {
			self.index -= 1;
		} else {
			self.index = self.titles.len() - 1;
		}
	}
	pub fn append_input(&mut self, user_char: char) {
		self.input.insert(self.input.len(), user_char);
	}
	pub fn delete_from_input(&mut self) {
		self.input.pop();
	}
	pub fn update_comments(&mut self, val: Vec<serde_json::Value>, index: usize) {
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
	pub fn change_focus(&mut self) {
		self.current_focus = (self.current_focus + 1) % 2;
	}
}

pub fn run_app<B: Backend>(
	terminal: &mut Terminal<B>,
	mut app: App,
	mut v: Vec<serde_json::Value>,
	mut last_post_id: &mut String,
) -> io::Result<()> {
	loop {
		terminal.draw(|f| ui(f, &mut app, &mut v))?;
		let reddit_cookie = app.cookie.clone();
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
						v = utils::get_posts(
							app.input.clone(),
							false,
							app.sort_by.clone(),
							&mut last_post_id,
							reddit_cookie,
						);
					}
					KeyCode::Char('b') => {
						app.sort_by = "best".to_string();
						v = utils::get_posts(
							app.input.clone(),
							false,
							app.sort_by.clone(),
							&mut last_post_id,
							reddit_cookie,
						);
					}
					KeyCode::Char('c') => {
						app.sort_by = "controversial".to_string();
						v = utils::get_posts(
							app.input.clone(),
							false,
							app.sort_by.clone(),
							&mut last_post_id,
							reddit_cookie,
						);
					}
					KeyCode::Right => {
						app.update_comments(v.clone(), app.items.state.selected().unwrap());
						app.next();
					}
					KeyCode::Down => {
						app.items.next();
						if app.items.state.selected() == Some(app.items.items.len()) {
							v = utils::get_posts(
								app.input.clone(),
								true,
								app.sort_by.clone(),
								&mut last_post_id,
								reddit_cookie,
							);
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
							v = utils::get_posts(
								app.input.clone(),
								true,
								app.sort_by.clone(),
								&mut last_post_id,
								reddit_cookie,
							);
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
						v = utils::get_posts(
							app.input.clone(),
							false,
							app.sort_by.clone(),
							&mut last_post_id,
							reddit_cookie,
						);
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
