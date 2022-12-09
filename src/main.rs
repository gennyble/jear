mod html;
mod notebook;
mod words;

use std::{fs::File, io::Write, ops::Deref};

use bempline::{Document, Options};
use camino::{Utf8Path, Utf8PathBuf};
use notebook::Notebook;
use words::WordsThing;

const WORDS: &'static str = "words";

pub struct NybleRoot(Utf8PathBuf);

impl Deref for NybleRoot {
	type Target = Utf8PathBuf;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl NybleRoot {
	pub fn words(&self) -> Utf8PathBuf {
		self.join(WORDS)
	}
}

pub struct Output(Utf8PathBuf);

impl Deref for Output {
	type Target = Utf8PathBuf;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Output {
	pub fn words(&self) -> Utf8PathBuf {
		let path = self.join(WORDS);

		if !path.exists() {
			std::fs::create_dir(&path).unwrap();
		}

		path
	}
}

fn main() {
	let nyble_root = match std::env::args().nth(1) {
		None => {
			eprintln!("First argument is nyble-root!");
			return;
		}
		Some(string_root) => NybleRoot(Utf8PathBuf::from(string_root)),
	};

	let output = match std::env::args().nth(2) {
		None => {
			eprintln!("The second argument is the output!");
			return;
		}
		Some(string_out) => Output(Utf8PathBuf::from(string_out)),
	};

	// Some bempline that we want to compile
	let files = vec!["about.html", "index.html"];
	for file in files {
		let doc = Document::from_file(nyble_root.join(file), Options::default()).unwrap();
		let string = doc.compile();
		let mut file = File::create(output.join(file)).unwrap();
		file.write_all(string.as_bytes()).unwrap();
	}

	// Ahhhh copy the directories ahhh
	copy_across(nyble_root.join("styles"), output.join("styles"));
	copy_across(nyble_root.join("media"), output.join("media"));

	// Special notebook handling
	Notebook::new(
		nyble_root.join("notebook.html"),
		nyble_root.join("notebook"),
	)
	.output(output.join("notebook.html"));

	// Special Words handling
	WordsThing::new(&nyble_root).output(&output);
}

pub fn copy_across<F: AsRef<Utf8Path>, T: AsRef<Utf8Path>>(from: F, to: T) {
	let from = from.as_ref();
	let to = to.as_ref();

	if !to.exists() {
		std::fs::create_dir_all(&to).expect("Faield to create path");
	}

	for entry in from.read_dir_utf8().expect("Failed readdir") {
		let entry = entry.unwrap();
		let name = entry.file_name();
		let meta = entry.metadata().unwrap();

		if meta.is_file() {
			std::fs::copy(entry.path(), to.join(name)).expect("Failed file copy");
		} else if meta.is_dir() {
			copy_across(entry.path(), to.join(name));
		} else if meta.is_symlink() {
			eprintln!("We don't use symlinks; what happened?");
			continue;
		} else {
			eprintln!("What even got here");
			continue;
		}
	}
}
