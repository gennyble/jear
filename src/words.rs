use core::fmt;
use std::{fs::File, io::Write};

use bempline::{Document, Options};
use camino::{Utf8Path, Utf8PathBuf};
use confindent::{Confindent, Value};
use time::{format_description::FormatItem, macros::format_description, Date};

use crate::{html, NybleRoot, Output};

const DATE: &[FormatItem] = format_description!("[year]-[month]-[day]");

pub struct WordsThing {
	words_list_template: Document,
	words_post_template: Document,
	all_the_words: Vec<Words>,
}

impl WordsThing {
	pub fn new(root: &NybleRoot) -> Self {
		let path = root.words();
		let words_post_template_path = path.join("words.html");
		let words_conf_path = path.join("words.conf");

		let words_post_template =
			Document::from_file(words_post_template_path, Options::default()).unwrap();
		let words_conf = Confindent::from_file(words_conf_path).unwrap();
		let all_the_words = words_conf
			.children("Words")
			.into_iter()
			.map(|v| Words::from_confindent(v, &path))
			.collect();

		let words_list_template =
			Document::from_file(root.join("words.html"), Options::default()).unwrap();

		Self {
			words_list_template,
			words_post_template,
			all_the_words,
		}
	}

	pub fn output(mut self, output: &Output) {
		// We want to do these in reverse so they appear newest-first in the list
		self.all_the_words
			.sort_by(|a, b| a.date.cmp(&b.date).reverse());

		for word in self.all_the_words {
			let page_title = format!("{} | nyble.dev", word.title);

			let mut doc = self.words_post_template.clone();
			doc.set("title", page_title);
			doc.set("words_title", &word.title);
			doc.set("words_content", word.content);

			match word.original {
				None => (),
				Some(o) => doc.set("original", o),
			}

			let content = doc.compile();

			let mut words_filename = Utf8PathBuf::from(word.filename);
			words_filename.set_extension("html");

			let words_out = output.words().join(&words_filename);

			let mut file = File::create(words_out).unwrap();
			file.write_all(content.as_bytes()).unwrap();

			let mut pattern = self.words_list_template.get_pattern("words").unwrap();
			pattern.set("date", word.date.format(DATE).unwrap());
			pattern.set("link", words_filename);
			pattern.set("title", word.title);
			self.words_list_template.set_pattern("words", pattern);
		}

		let words_list_path = output.words().join("index.html");
		let content = self.words_list_template.compile();

		let mut file = File::create(words_list_path).unwrap();
		file.write_all(content.as_bytes()).unwrap();
	}
}

pub struct Words {
	filename: String,
	title: String,
	date: Date,
	content: String,
	original: Option<Original>,
}

impl Words {
	pub fn from_confindent<P: AsRef<Utf8Path>>(value: &Value, root_path: P) -> Self {
		let words_name = value.value_owned().unwrap();
		let title = value.child_owned("Title").unwrap();
		let date = Date::parse(&value.child_owned("Date").unwrap(), DATE).unwrap();
		let original = value
			.child_owned("Original")
			.map(|link| Original::new(link));

		let words_path = root_path.as_ref().join(&words_name);
		let content = html::file_html(words_path);

		Self {
			filename: words_name,
			title,
			date,
			content,
			original,
		}
	}
}

pub struct Original {
	link: String,
	from: Place,
}

impl Original {
	pub fn new(link: String) -> Self {
		let from = if link.starts_with("https://cohost.org") {
			Place::Cohost
		} else {
			Place::Unknown
		};

		Self { link, from }
	}
}

impl fmt::Display for Original {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.from {
			Place::Cohost => write!(f, "<a href=\"{}\">on cohost</a>", self.link),
			Place::Unknown => write!(
				f,
				"here: <a href=\"{}\">{}</a>",
				self.link,
				html::htmlspecialchars(&self.link)
			),
		}
	}
}

pub enum Place {
	Unknown,
	Cohost,
}
