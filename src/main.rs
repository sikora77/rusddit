mod app;
mod tabs;
mod utils;

use crate::app::App;

use crossterm::{
	event::{DisableMouseCapture, EnableMouseCapture},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, fs, io, path::Path};
use tui::{
	backend::{Backend, CrosstermBackend},
	style::{Color, Modifier, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Tabs},
	Frame, Terminal,
};

static mut LAST_POST_ID: String = String::new();

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = std::env::args().collect();
	for i in 0..args.len() {
		let path = Path::join(
			home::home_dir().expect("what").as_path(),
			Path::new(".config/rusddit/"),
		);
		if args[i] == "-c" || args[i] == "--cookie" {
			if Path::exists(Path::join(path.as_path(), "cookie.txt").as_path()) == false {
				match fs::create_dir_all(path.as_path()) {
					Ok(_x) => {
						let _file = std::fs::File::create(Path::join(path.as_path(), "cookie.txt"))
							.expect("Shoouldnt happen");
						let _res = fs::write(
							Path::join(path.as_path(), "cookie.txt"),
							args[i + 1].clone(),
						);
					}
					Err(_x) => {
						println!("{}", "Couldn't create cookie file")
					}
				}
			}
		}
	}
	let m: Vec<serde_json::Value>;
	unsafe {
		m = utils::get_posts("".to_owned(), false, "hot".to_string(), &mut LAST_POST_ID);
	}

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	let app = App::new();
	// app.update_comments(m.clone(), 0);
	let res;
	unsafe {
		res = app::run_app(&mut terminal, app, m, &mut LAST_POST_ID);
	}

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
		0 => tabs::draw_first_tab(f, &mut app, v, tabs),
		1 => tabs::draw_second_tab(f, &mut app, v, tabs),
		2 => tabs::draw_third_tab(f, &mut app, tabs),
		_ => unreachable!(),
	};
}
