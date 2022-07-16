use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
	pub tabs: Vec<String>,
	pub cookie: String,
}
impl UserConfig {
	pub fn new() -> UserConfig {
		UserConfig {
			tabs: vec![],
			cookie: "".to_string(),
		}
	}
	pub fn readConfig(&mut self) {
		let config_string = match std::fs::read_to_string(Path::join(
			home::home_dir().expect("what").as_path(),
			Path::new(".config/rusddit/config.txt"),
		)) {
			Ok(x) => x,
			Err(_x) => {
				println!("Couldn't find cookie file. Starting an anonymous session");
				"".to_string()
			}
		};
		if (config_string == "".to_string()) {
			self.cookie = "".to_string();
			self.tabs = vec![];
			return;
		}
		let config: UserConfig = serde_json::from_str(&config_string).unwrap_or(UserConfig {
			tabs: vec![],
			cookie: "".to_string(),
		});
		self.cookie = config.cookie;
		self.tabs = config.tabs;
		return;
	}
	pub fn changeConfig(&mut self, new_cookie: Option<String>, new_tabs: Option<Vec<String>>) {
		self.cookie = new_cookie.unwrap_or(self.cookie.clone());
		self.tabs = new_tabs.unwrap_or(self.tabs.clone());
		match serde_json::to_string(self) {
			Ok(x) => fs::write(
				Path::join(
					home::home_dir().expect("what").as_path(),
					Path::new(".config/rusddit/config.txt"),
				),
				x,
			)
			.unwrap(),
			Err(_x) => {
				println!("failed to change config")
			}
		}
	}
}
