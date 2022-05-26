fn main() {
	use viuer::{print_from_file, Config};

	let conf = Config {
		width: Some(40),
		height: Some(20),
		x: 50,
		y: 0,
		use_sixel: true,
		..Default::default()
	};
	// will resize the image to fit in 40x30 terminal cells and print it
	print_from_file("img.png", &conf).expect("Image printing failed.");
	println!("Hey");
}
