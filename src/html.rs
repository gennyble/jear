use std::collections::HashMap;

use quark::{Inline, Link, Parser, Token};

pub fn parser_html(parser: Parser) -> String {
	tokens_html(parser.tokens(), &parser.references)
}

pub fn tokens_html<'a, I>(tokens: I, refs: &HashMap<String, String>) -> String
where
	I: Iterator<Item = &'a Token>,
{
	let mut ret = String::new();

	for tok in tokens {
		match tok {
			Token::Header { level, inner } => {
				if *level > 5 {
					eprintln!(
						"Header level greater than 5, lol. Making you a good little <h{level}> tag"
					);
				}
				let text = inlines_html(&inner, refs);
				ret.push_str(&format!("<h{level}>{text}</h{level}>"));
			}
			Token::Paragraph { inner } => {
				ret.push_str(&format!("<p>{}</p>", inlines_html(&inner, refs)));
			}
			Token::CodeBlock { lang, code } => {
				ret.push_str(&format!("<pre><code>{code}</pre></code>"));
			}
		}
	}

	ret
}

pub fn inlines_html(inlines: &[Inline], refs: &HashMap<String, String>) -> String {
	let mut ret = String::new();

	for inl in inlines {
		let str =
			match inl {
				Inline::Break => "<br>".to_owned(),
				Inline::Text(txt) => txt.clone(),
				Inline::Code(code) => format!("<code>{code}</code>"),
				Inline::Interlink(inter) => {
					let Link { name, location } = inter;
					eprintln!(
						"No interlnks! name={} link={}",
						name.as_deref().unwrap_or("None".into()),
						location
					);

					match name {
						None => format!("{{{location}}}"),
						Some(name) => format!("{{{name}|{location}}}"),
					}
				}
				Inline::Link(link) => {
					let Link { name, location } = link;
					let name = name
						.as_deref()
						.map(<_>::to_owned)
						.unwrap_or_else(|| htmlspecialchars(location));
					format!("<a href=\"{location}\">{name}</a>")
				}
				Inline::ReferenceLink(link) => {
					let Link { name, location } = link;

					match refs.get(location) {
						None => {
							eprintln!("Failed to resolve a reflink: {location}. Outputting as regular text");
							format!("{location}")
						}
						Some(real_location) => {
							let name = name
								.as_deref()
								.map(<_>::to_owned)
								.unwrap_or_else(|| htmlspecialchars(location));
							format!("<a href=\"{real_location}\">{name}</a>")
						}
					}
				}
			};

		ret.push_str(&str);
	}

	ret
}

// This is from turquoise lmfao
fn htmlspecialchars<S: AsRef<str>>(raw: S) -> String {
	let mut ret = String::new();

	for ch in raw.as_ref().chars() {
		match ch {
			'<' => ret.push_str("&lt;"),
			'>' => ret.push_str("&gt;"),
			'&' => ret.push_str("&amp;"),
			'\'' => ret.push_str("&apos;"),
			'"' => ret.push_str("&quot;"),
			c => ret.push(c),
		}
	}

	ret
}
