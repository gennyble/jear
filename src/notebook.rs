use std::{fs::File, io::Write};

use bempline::{Document, Options};
use camino::Utf8Path;
use quark::Parser;

use crate::html;

pub struct Notebook {
	template: Document,
	pages: Vec<NotebookPage>,
}

impl Notebook {
	pub fn new<P: AsRef<Utf8Path>>(template: P, path: P) -> Self {
		let mut pages = vec![];

		let readdir = path.as_ref().read_dir_utf8();
		for entry in readdir.expect("Failed to read Notebook dir") {
			let entry = entry.expect("Failed to read notebook entry");
			pages.push(NotebookPage::from_file(entry.path()));
		}

		let template = Document::from_file(template.as_ref(), Options::default()).unwrap();

		Self { template, pages }
	}

	pub fn output<P: AsRef<Utf8Path>>(mut self, path: P) {
		let mut file = File::create(path.as_ref()).expect("Failed to create Notebook output");

		let pattern = self
			.template
			.get_pattern("page")
			.expect("Notebook Page pattern not found");

		for page in self.pages {
			let mut pat = pattern.clone();
			pat.set("content", page.content);
			self.template.set_pattern("page", pat);
		}

		file.write_all(self.template.compile().as_bytes())
			.expect("Failed to write out notebook!")
	}
}

pub struct NotebookPage {
	name: String,
	content: String,
}

impl NotebookPage {
	pub fn from_file<P: AsRef<Utf8Path>>(path: P) -> Self {
		let path = path.as_ref();
		let name = path.file_name().unwrap().to_owned();
		let file_content = std::fs::read_to_string(path).expect("Failed to read NotebookPage");

		let mut qp = Parser::new();
		qp.parse(file_content);
		let content = html::parser_html(qp);
		NotebookPage { name, content }
	}
}
