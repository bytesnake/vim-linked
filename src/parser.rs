use std::path::PathBuf;
use std::collections::HashMap;
use pulldown_cmark::{Parser, Tag, HeadingLevel, Event};
use miniserde::{json, Serialize, Deserialize};

use crate::error::{Result, Error};

pub type NodeId = String;

#[derive(Debug, Deserialize)]
pub enum JumpMode {
    Forward,
    Backward,
    ForwardEnd,
    BackwardEnd
}

#[derive(Debug, Deserialize)]
pub struct JumpTo {
    mode: JumpMode,
    cursor: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Link {
    path: Option<PathBuf>,
    note: Option<NodeId>,
    text: Option<String>
}

impl Link {
    pub fn empty() -> Link {
        Link {
            path: None,
            note: None,
            text: None
        }
    }

    pub fn from_str(input: &str) -> Result<Link> {
        let mut link = Link::empty();

        if input.len() == 0 {
            return Err(Error::InvalidLink(input.into(), "Empty query".into()));
        } else if input.matches("@").count() > 1 {
            return Err(Error::InvalidLink(input.into(), "More than one `@` seperator in link".into()));
        } else if input.matches("#").count() > 1 {
            return Err(Error::InvalidLink(input.into(), "More than one `#` seperator in link".into()));
        }

        // check if link contains note id
        if input.contains("@") {
            let elms = input.splitn(2, "@").collect::<Vec<_>>();

            if elms[0].len() > 0 {
                link.path = Some(PathBuf::from(elms[0]));
            }
            // check for text search
            if elms[1].contains("#") {
                let elms = elms[1].splitn(2, "#").collect::<Vec<_>>();

                link.note = Some(elms[0].into());
                link.text = Some(elms[1].into());
            } else {
                link.note = Some(elms[1].into());
            }
        } else if input.contains("#") {
            // if it doesn't contain a note ID, check for path with text seperator "#"
            let elms = input.splitn(2, "#").collect::<Vec<_>>();

            if elms[0].len() > 0 {
                link.path = Some(PathBuf::from(elms[0]));
            }
            link.text = Some(elms[1].into());
        } else {
            // if it doesn't use a text seperator, it just points to a local file
            if input.len() > 0 {
                link.path = Some(PathBuf::from(input));
            }
        }

        Ok(link)
    }

    pub fn to_string(mut self) -> String {
        let mut out = String::new();
        if let Some(ref mut path) = self.path {
            out.push_str(path.to_str().unwrap());
        }

        out
    }


    pub fn is_valid(input: &str) -> bool {
        Self::from_str(input).is_ok()
    }
}

fn parse_header(input: &str) -> Option<(String, String)> {
    let parts = input.split("-").collect::<Vec<_>>();
    
    match parts[..] {
        [a, b] => Some((a.trim().to_string(), b.trim().to_string())),
        _ => None
    }
}

pub struct Parse {
    nodes: HashMap<NodeId, (String, usize, Vec<Link>)>,
    backlinks: HashMap<Link, Vec<NodeId>>,
    content: Vec<String>,
}

impl Parse {
    pub fn new() -> Parse {
        Parse {
            nodes: HashMap::new(),
            backlinks: HashMap::new(),
            content: Vec::new(),
        }
    }

    pub fn update_content(&mut self, content: &str) -> Result<()> {
        let mut source = Parser::new(content).into_offset_iter();

        let mut nodes = HashMap::new();
        let mut backlinks = HashMap::new();

        let mut current_node: Option<(NodeId, (String, usize, Vec<Link>))> = None;

        while let Some((elm, range)) = source.next() {
            match elm {
                Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                    if let Some((Event::Text(title),_)) = source.next() {
                        let line_num = content[..range.start].matches("\n").count();

                        if let Some((id, title)) = parse_header(&title) {
                            current_node = Some((id, (title, line_num, Vec::new())));
                        } else {
                            current_node = None;
                        }
                    }
                },
                Event::End(Tag::Heading(HeadingLevel::H1, _, _)) => {
                    if let Some(current_node) = current_node.take() {
                        // insert backlinks
                        for link in &current_node.1.2 {
                            // we don't save backlinks to individual text section within a single
                            // note
                            let mut link = link.clone();
                            link.text = None;

                            backlinks.entry(link).or_insert(Vec::new())
                                .push(current_node.0.to_string());
                        }

                        nodes.insert(current_node.0, current_node.1);
                    }
                },
                Event::Start(Tag::Link(_, link, _)) => {
                    if let Some(ref mut elm) = current_node {
                        let link = Link::from_str(&link[..])?;
                        elm.1.2.push(link);
                    }
                },
                _ => {}
            }
        }
    
        self.nodes = nodes;
        self.backlinks = backlinks;
        self.content = content.lines().map(|x| x.to_string()).collect();

        Ok(())
    }

    pub fn go_to(&mut self, infos: &str) -> Result<String> {
        let jump_to: JumpTo = json::from_str(infos).unwrap();
        let (line, shift) = (&jump_to.cursor[1], &jump_to.cursor[2]);

        // first check if we have a link
        let mut link: Option<Link> = None;
        let line: &str = &self.content[*line as usize - 1];
        for (elm, range) in Parser::new(line).into_offset_iter() {
            match elm {
                Event::Start(Tag::Link(_, content, _)) => {
                    if !range.contains(&(*shift as usize)) {
                        continue;
                    }

                    link = Some(Link::from_str(&content[..])?);
                    break;
                },
                _ => {}
            }
        }

        match (jump_to.mode, link) {
            (JumpMode::Forward, Some(Link { note, .. })) => {
                let note = note.unwrap();
                match self.nodes.get(&note) {
                    Some(ref node) => Ok(format!("{{ \"line\": {}}}", node.1 + 1)),
                    None => Err(Error::MissingNote(format!("id {} not found", note))),
                }
            },
            (JumpMode::ForwardEnd, None) => {
                return Ok("".into());
            },
            x => panic!("call mode {:?} not supported!", x)
        }
    }
}

mod tests {
    use super::{Link, Result, Error};
    use std::path::PathBuf;

    fn link(path: Option<&str>, note: Option<&str>, text: Option<&str>) -> Link {
        let path = path.map(|x| PathBuf::from(x));
        let note = note.map(|x| x.to_string());
        let text = text.map(|x| x.to_string());

        Link {
            path, note, text
        }
    }

    fn invalid_link(query: &str, error: &str) -> Result<Link> {
        Err(Error::InvalidLink(query.into(), error.into()))
    }

    #[test]
    fn link_parsing() {
        let samples = &[
            "/tmp/test.md@fads79#blaa",
            "@jfkl3dk23#this text",
            "#this text",
            "/books/murphys.md#Sentence beginning",
        ];

        let success: &[Result<Link>] = &[
            Ok(link(Some("/tmp/test.md"), Some("fads79"), Some("blaa"))),
            Ok(link(None, Some("jfkl3dk23"), Some("this text"))),
            Ok(link(None, None, Some("this text"))),
            Ok(link(Some("/books/murphys.md"), None, Some("Sentence beginning"))),
        ];

        let samples_err = &[
            "",
            "@@fdsakl3",
            "/blub/@fjdsakl##432",
        ];

        let fails = &[
            invalid_link("", "Empty query"),
            invalid_link("@@fdsakl3", "More than one `@` seperator in link"),
            invalid_link("/blub/@fjdsakl##432", "More than one `#` seperator in link"),
        ];

        for (sample, expected) in samples.into_iter().zip(success.into_iter()) {
            let link = Link::from_str(sample);
            assert_eq!(&link, expected);
        }

        for (sample, expected) in samples_err.into_iter().zip(fails.into_iter()) {
            let link = Link::from_str(sample);
            assert_eq!(&link, expected);
            assert!(!&Link::is_valid(sample));
        }
    }
}
