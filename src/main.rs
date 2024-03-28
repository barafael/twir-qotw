use std::{ops::Not, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(short, long, default_value = "content")]
    input: PathBuf,

    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Quote {
    content: String,
    //date: String,
    filename: PathBuf,
}

fn main() {
    let args = Arguments::parse();

    let mut quotes = Vec::new();
    for entry in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_file().not() {
            continue;
        }
        let Some(filename) = entry.file_name().to_str() else {
            continue;
        };
        if filename.ends_with(".md").not() && filename.ends_with(".markdown").not() {
            continue;
        }
        println!("{filename}");

        let contents = std::fs::read_to_string(entry.path()).unwrap();
        let Some(quote) = find_quote(&contents, entry.path().to_owned()) else {
            continue;
        };
        quotes.push(quote);
    }

    let mut output = String::new();
    output.push_str("<h1>Quotes of all the Weeks</h1>\n");
    for Quote { content, filename } in quotes {
        output.push_str(&format!("<h2>{}</h2>\n", filename.display()));
        output.push_str(&content);
        output.push('\n');
    }
    std::fs::write(args.output, output).unwrap();
}

fn find_quote(contents: &str, filename: PathBuf) -> Option<Quote> {
    let parser = pulldown_cmark::Parser::new(contents);

    let mut inside_quote = false;

    let mut events = Vec::new();
    use pulldown_cmark::Event::*;
    for event in parser {
        match event {
            Text(t) if t == "Quote of the Week".into() => {
                inside_quote = true;
            }
            Text(t) if inside_quote && t == "Thanks to ".into() => {
                break;
            }
            Text(t) if inside_quote && t == "Submit your quotes for next week".into() => {
                break;
            }
            Text(t)
                if inside_quote && t == "Please submit quotes and vote for next week!".into() =>
            {
                break;
            }
            a if inside_quote => {
                events.push(a);
            }
            _ => {}
        }
    }

    //let options = pulldown_cmark_to_cmark::Options::default();

    if events.is_empty() {
        return None;
    }
    let mut events = events.into_iter();
    events.next();
    //let _state = cmark_with_options(events, &mut buf, options).unwrap();

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, events);

    Some(Quote {
        content: html_output,
        filename,
    })
}
