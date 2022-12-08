mod html;
mod notebook;

use std::{fs::File, io::Write};

use bempline::{Document, Options};
use camino::{Utf8Path, Utf8PathBuf};
use notebook::Notebook;
use quark::Parser;

fn main() {
	let nyble_root = match std::env::args().nth(1) {
		None => {
			eprintln!("First argument is nyble-root!");
			return;
		}
		Some(string_root) => Utf8PathBuf::from(string_root),
	};

	let output = match std::env::args().nth(2) {
		None => {
			eprintln!("The second argument is the output!");
			return;
		}
		Some(string_out) => Utf8PathBuf::from(string_out),
	};

	let files = vec!["about.html", "index.html"];

	for file in files {
		std::fs::copy(nyble_root.join(file), output.join(file)).expect("Failed to copy file");
	}

	copy_across(nyble_root.join("styles"), output.join("styles"));
	copy_across(nyble_root.join("media"), output.join("media"));

	Notebook::new(
		nyble_root.join("notebook.html"),
		nyble_root.join("notebook"),
	)
	.output(output.join("notebook.html"));
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
