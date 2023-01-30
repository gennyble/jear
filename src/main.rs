mod html;
mod notebook;
mod words;

use std::{fs::File, io::Write, ops::Deref};

use bempline::{Document, Options};
use camino::{Utf8Path, Utf8PathBuf};
use confindent::Confindent;
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
	let conf = match std::env::args().nth(1) {
		None => {
			eprintln!("The first argument must be path to the config");
			return;
		}
		Some(string) => Confindent::from_file(string).unwrap(),
	};
	let JearConfig {
		nyble_root,
		output,
		bempline_build,
		copy,
		symlink,
		silly_gifs,
		silly_gifs_sym,
	} = JearConfig::make(conf).unwrap();

	println!(
		"NybleRoot {}\nOutput {}\nSillyGifs {silly_gifs}\n\tSymlink {silly_gifs_sym}",
		nyble_root.deref(),
		output.deref()
	);

	// Some bempline that we want to compile
	for file in bempline_build {
		let doc = Document::from_file(nyble_root.join(&file), Options::default()).unwrap();
		let string = doc.compile();
		let mut file = File::create(output.join(file)).unwrap();
		file.write_all(string.as_bytes()).unwrap();
	}

	for path in copy {
		copy_across(nyble_root.join(&path), output.join(&path), false);
	}

	for path in symlink {
		std::os::unix::fs::symlink(nyble_root.join(&path), output.join(&path)).unwrap();
	}

	// Special SillyGif handling. They're so large that we want to symlink
	let sillyout = output.join("media").join("sillygifs");
	if !sillyout.exists() && silly_gifs_sym {
		std::os::unix::fs::symlink(silly_gifs, sillyout).unwrap();
	} else if !silly_gifs_sym {
		copy_across(silly_gifs, sillyout, true);
	}

	// Special notebook handling
	Notebook::new(
		nyble_root.join("notebook.html"),
		nyble_root.join("notebook"),
	)
	.output(output.join("notebook.html"));

	// Special Words handling
	WordsThing::new(&nyble_root).output(&output);

	copy_across(
		nyble_root
			.words()
			.join("gif-selfies-and-color-quantization"),
		output.words().join("gif-selfies-and-color-quantization"),
		false,
	)
}

pub fn copy_across<F: AsRef<Utf8Path>, T: AsRef<Utf8Path>>(from: F, to: T, hardlink: bool) {
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
			if !hardlink {
				std::fs::copy(entry.path(), to.join(name)).expect("Failed file copy");
			} else {
				let fullto = to.join(name);
				if !fullto.exists() {
					std::fs::hard_link(entry.path(), fullto).expect("Failed to hardlink file");
				}
			}
		} else if meta.is_dir() {
			copy_across(entry.path(), to.join(name), hardlink);
		} else if meta.is_symlink() {
			eprintln!("We don't use symlinks; what happened?");
			continue;
		} else {
			eprintln!("What even got here");
			continue;
		}
	}
}

pub struct JearConfig {
	nyble_root: NybleRoot,
	output: Output,
	/// Just files that need to be run through bempline so they can be compiled
	bempline_build: Vec<Utf8PathBuf>,
	copy: Vec<Utf8PathBuf>,
	symlink: Vec<Utf8PathBuf>,
	silly_gifs: Utf8PathBuf,
	// Whether or not to use symlinks.
	// true: use symlinks
	// false: use hardlinks
	silly_gifs_sym: bool,
}

impl JearConfig {
	pub fn make(c: Confindent) -> Result<Self, ()> {
		let nyble_root = match c.child_value("NybleRoot") {
			None => {
				eprintln!("Missing NybleRoot");
				return Err(());
			}
			Some(string_root) => NybleRoot(Utf8PathBuf::from(string_root)),
		};

		let output = match c.child_value("Output") {
			None => {
				eprintln!("Missing Output");
				return Err(());
			}
			Some(string_out) => Output(Utf8PathBuf::from(string_out)),
		};

		let bempline_build = c
			.children("Build")
			.into_iter()
			.map(|child| Utf8PathBuf::from(child.value().unwrap()))
			.collect();

		let copy = c
			.children("Copy")
			.into_iter()
			.map(|child| Utf8PathBuf::from(child.value().unwrap()))
			.collect();

		let symlink = c
			.children("Symlink")
			.into_iter()
			.map(|child| Utf8PathBuf::from(child.value().unwrap()))
			.collect();

		let silly_gifs = match c.child_parse("SillyGifs") {
			Ok(u) => u,
			Err(_) => {
				eprintln!("Missing or malformed SillyGifs");
				return Err(());
			}
		};

		let silly_gifs_sym = match c.child("SillyGifs").unwrap().child_value("Symlink") {
			None => {
				eprintln!("SillyGifs.Symlink is required. true for symlink, false for hardlink");
				return Err(());
			}
			Some(v) if v.to_lowercase() == "false" => false,
			Some(v) if v.to_lowercase() == "true" => true,
			Some(un) => {
				eprintln!("SillyGifs.Symlink value '{un}' is confusing");
				return Err(());
			}
		};

		Ok(Self {
			nyble_root,
			output,
			bempline_build,
			copy,
			symlink,
			silly_gifs,
			silly_gifs_sym,
		})
	}
}
