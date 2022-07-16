use std::path::Path;

pub fn get_posts(
	input: String,
	before: bool,
	sort_by: String,
	mut last_post_id: &mut String,
	reddit_cookie: String,
) -> Vec<serde_json::Value> {
	let slash = match input.as_str() {
		"" => "",
		_ => "/",
	};
	let mut url = format!(
		"{}{}{}{}{}",
		"https://reddit.com/", input, slash, sort_by, ".json?limit=100"
	);
	if before {
		url = format!("{}{}{}", url, "&after=", last_post_id.clone())
	}

	let client_builder: reqwest::blocking::ClientBuilder;
	if reddit_cookie != "" {
		let cookie = &format!(
			"{}{}{}",
			"reddit_session=", reddit_cookie, "; Domain=reddit.com"
		);
		let cookie_url = "https://reddit.com".parse::<reqwest::Url>().unwrap();

		let jar = reqwest::cookie::Jar::default();
		jar.add_cookie_str(cookie, &cookie_url);
		let cookie_store = std::sync::Arc::new(jar);

		client_builder = reqwest::blocking::Client::builder()
			.cookie_store(true)
			.cookie_provider(cookie_store);
	} else {
		client_builder = reqwest::blocking::Client::builder()
	}
	let client = client_builder.build().expect("Fuuuck");
	let req = client.get(&url).build().expect("fuck");
	let client_body = client.execute(req).expect("client request failed");
	let body = match client_body.text() {
		Ok(x) => x,
		Err(err) => {
			panic!("called `Result::unwrap()` on an `Err` value: {:?}", err)
		}
	};
	return filter_out_text_posts(
		match serde_json::from_str(&body) {
			Ok(x) => x,
			Err(x) => panic!("{:?}", x),
		},
		&mut last_post_id,
	)
	.to_owned();
}

pub fn filter_out_text_posts(
	v: serde_json::Value,
	last_post_id: &mut String,
) -> Vec<serde_json::Value> {
	let ammount = match v["data"]["dist"].as_i64() {
		Some(x) => x,
		None => 0,
	};
	*last_post_id = v["data"]["children"][ammount as usize - 1]["data"]["name"]
		.as_str()
		.expect("msg")
		.clone()
		.to_string();

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
