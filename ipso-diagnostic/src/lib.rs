use std::str::from_utf8;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};
use std::{fs::File, io};

mod test;

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum Source {
    File { path: PathBuf },
    Interactive { label: String },
}

impl Source {
    pub fn to_str(&self) -> &str {
        match self {
            Source::File { path } => path.to_str().unwrap(),
            Source::Interactive { label } => label,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Location {
    pub source: Source,
    pub offset: usize,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Message {
    pub content: String,
    pub addendum: Option<String>,
}

pub struct Diagnostic {
    items: Vec<Message>,
    located_items: Vec<(Location, Message)>,
}

impl Diagnostic {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Diagnostic {
            items: Vec::new(),
            located_items: Vec::new(),
        }
    }

    pub fn item(&mut self, location: Option<Location>, message: Message) {
        match location {
            None => self.items.push(message),
            Some(location) => {
                match self
                    .located_items
                    .binary_search_by_key(&location.offset, |i| i.0.offset)
                {
                    Err(ix) => self.located_items.insert(ix, (location, message)),
                    Ok(ix) => self.located_items.insert(ix + 1, (location, message)),
                }
            }
        }
    }

    pub fn report_error_heading(path: &str, line: usize, col: usize, message: &str) -> String {
        format!("{}:{}:{}: error: {}", path, line, col, message)
    }

    pub fn report_located_message(
        line: usize,
        col: usize,
        path: &str,
        line_str: &str,
        message: &Message,
    ) -> String {
        let mut result = String::new();
        let caret: String = {
            let mut caret: String = " ".repeat(col - 1);
            caret.push('^');
            caret
        };
        let line1 = Self::report_error_heading(path, line, col, &message.content);
        let pad_amount = ((line as f32).log(10.0).floor() as usize) + 1;
        let padding: String = " ".repeat(pad_amount);

        let line2 = format!("{} |", padding);
        let line3 = format!("{} | {}", line, line_str);
        let line4 = format!("{} | {}", padding, caret);

        result.push_str(&line1);
        result.push('\n');
        result.push_str(&line2);
        result.push('\n');
        result.push_str(&line3);
        result.push('\n');
        result.push_str(&line4);
        match &message.addendum {
            None => {}
            Some(addendum) => {
                result.push('\n');
                result.push_str(addendum.as_str());
            }
        }
        result
    }

    pub fn report_all(self) -> io::Result<()> {
        struct FileEntry {
            file: BufReader<File>,
            line_str: String,
            line: usize,
            offset: usize,
        }
        enum LocationEntry {
            InteractiveEntry { label: String },
            FileEntry(FileEntry),
        }

        fn get_entry(
            files: &mut HashMap<Source, LocationEntry>,
            source: Source,
        ) -> io::Result<&mut LocationEntry> {
            Ok(files
                .entry(source)
                .or_insert_with_key(|location| match location {
                    Source::Interactive { label } => LocationEntry::InteractiveEntry {
                        label: label.clone(),
                    },
                    Source::File { path } => {
                        let file = File::open(path).unwrap();
                        let file = BufReader::new(file);
                        let line_str = String::new();
                        let line: usize = 0;
                        let offset: usize = 0;
                        LocationEntry::FileEntry(FileEntry {
                            file,
                            line_str,
                            line,
                            offset,
                        })
                    }
                }))
        }

        for message in self.items {
            let result = format!("error: {}", message.content);

            match io::stderr().write(result.as_bytes()) {
                Ok(_) => {}
                Err(err) => return Err(err),
            };
            match io::stderr().write(b"\n") {
                Ok(_) => {}
                Err(err) => return Err(err),
            };
        }

        let mut source_map: HashMap<Source, LocationEntry> = HashMap::new();
        for (location, message) in self.located_items.into_iter() {
            let result = match get_entry(&mut source_map, location.source.clone()) {
                Err(err) => return Err(err),
                Ok(location_entry) => match location_entry {
                    LocationEntry::InteractiveEntry { label } => {
                        Self::report_error_heading(label, 1, location.offset, &message.content)
                    }
                    LocationEntry::FileEntry(file_entry) => {
                        let mut pos = location.offset;
                        while location.offset >= file_entry.offset {
                            pos -= file_entry.line_str.len();
                            file_entry.line_str.clear();
                            match file_entry.file.read_line(&mut file_entry.line_str) {
                                Err(err) => return Err(err),
                                Ok(bytes_read) => {
                                    if bytes_read == 0 {
                                        return Ok(());
                                    } else {
                                        file_entry.offset += bytes_read;
                                        file_entry.line += 1;
                                    }
                                }
                            }
                        }
                        let col: usize = {
                            let item_bytes = &(file_entry.line_str.as_bytes())[0..pos];
                            from_utf8(item_bytes).unwrap().chars().count() + 1
                        };
                        Diagnostic::report_located_message(
                            file_entry.line,
                            col,
                            location.source.to_str(),
                            file_entry.line_str.trim_end_matches('\n'),
                            &message,
                        )
                    }
                },
            };
            match io::stderr().write(result.as_bytes()) {
                Ok(_) => {}
                Err(err) => return Err(err),
            };
            match io::stderr().write(b"\n") {
                Ok(_) => {}
                Err(err) => return Err(err),
            };
        }
        Ok(())
    }
}
