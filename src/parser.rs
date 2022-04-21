use std::path::PathBuf;
use std::collections::HashMap;
use pulldown_cmark::{Parser, Tag, HeadingLevel, Event};
use miniserde::{json, Deserialize};

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

        if input.is_empty() {
            return Err(Error::InvalidLink(input.into(), "Empty query".into()));
        } else if input.matches('@').count() > 1 {
            return Err(Error::InvalidLink(input.into(), "More than one `@` seperator in link".into()));
        } else if input.matches('#').count() > 1 {
            return Err(Error::InvalidLink(input.into(), "More than one `#` seperator in link".into()));
        }

        // check if link contains note id
        if input.contains('@') {
            let elms = input.splitn(2, '@').collect::<Vec<_>>();

            if !elms[0].is_empty() {
                link.path = Some(PathBuf::from(elms[0]));
            }
            // check for text search
            if elms[1].contains('#') {
                let elms = elms[1].splitn(2, '#').collect::<Vec<_>>();

                link.note = Some(elms[0].into());
                link.text = Some(elms[1].into());
            } else {
                link.note = Some(elms[1].into());
            }
        } else if input.contains('#') {
            // if it doesn't contain a note ID, check for path with text seperator "#"
            let elms = input.splitn(2, '#').collect::<Vec<_>>();

            if !elms[0].is_empty() {
                link.path = Some(PathBuf::from(elms[0]));
            }
            link.text = Some(elms[1].into());
        } else {
            // if it doesn't use a text seperator, it just points to a local file
            if !input.is_empty() {
                link.path = Some(PathBuf::from(input));
            }
        }

        Ok(link)
    }

    //pub fn to_string(mut self) -> String {
    //    let mut out = String::new();
    //    if let Some(ref mut path) = self.path {
    //        out.push_str(path.to_str().unwrap());
    //    }

    //    out
    //}
}

fn parse_header(input: &str) -> Result<(String, String)> {
    let parts = input.splitn(2, '-').collect::<Vec<_>>();
    
    match parts[..] {
        [a, b] => Ok((a.trim().to_string(), b.trim().to_string())),
        _ => Err(Error::InvalidHeader(input.into())),
    }
}

#[derive(Debug)]
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

        let mut last_note: Option<String> = None;

        while let Some((elm, range)) = source.next() {
            match elm {
                Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                    if let Some((Event::Text(title),_)) = source.next() {
                        let line_num = content[..range.start].matches('\n').count();

                        let (id, title) = parse_header(&title)?;
                        last_note = Some(id.clone());
                        nodes.insert(id, (title, line_num, Vec::new()));

                    }
                },
                Event::Start(Tag::Link(_, link, _)) => {
                    if let Some(id) = &last_note {
                        let link = Link::from_str(&link[..])?;

                        // we don't save backlinks to individual text section within a single
                        // note
                        let mut link2 = link.clone();
                        link2.text = None;

                        backlinks.entry(link2).or_insert_with(Vec::new)
                            .push(id.to_string());

                        nodes.get_mut(&id.to_string()).unwrap().2.push(link);
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
                Event::Text(content) => {
                    if !content.starts_with('(') || !content.ends_with(')') {
                        continue;
                    }

                    link = Some(Link::from_str(&content[1..content.len()-1])?);
                    break;
                },
                _ => {}
            }
        }

        let (path, note, text) = match link {
            Some(Link { path, note, text}) => (path, note, text),
            None => return Ok(String::new())
        };

        match (jump_to.mode, path, note, text) {
            (JumpMode::Forward, _, Some(note), _) => {
                match self.nodes.get(&note) {
                    Some(node) => Ok(format!("{{ \"line\": {}}}", node.1 + 1)),
                    None => Err(Error::MissingNote(format!("id {} not found", note))),
                }
            },
            (JumpMode::Forward, Some(path), _, Some(text)) => {
                Ok(format!("{{ \"path\": \"{}\", \"text\": \"{}\"}}", path.to_str().unwrap(), text))
            },
            (mode, path, note, text) => {
                let err_msg = format!("Mode {:?} not supported with {:?} {:?} {:?}", mode, path, note, text);
                Err(Error::Other(err_msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Parse, Link, Result, Error};

    use std::path::PathBuf;

    fn link(path: Option<&str>, note: Option<&str>, text: Option<&str>) -> Link {
        let path = path.map(PathBuf::from);
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

        for (sample, expected) in samples.iter().zip(success.iter()) {
            let link = Link::from_str(sample);
            assert_eq!(&link, expected);
        }

        for (sample, expected) in samples_err.iter().zip(fails.iter()) {
            let link = Link::from_str(sample);
            assert_eq!(&link, expected);
            assert!(!Link::from_str(sample).is_ok());
        }
    }

    #[test]
    fn markdown_parsing() {
        let content = r"
# asdf - This is a sample note

Some text

More text *import text*

# ghjk - Second note

This [links](@asdf) to first one";

        let mut parser = Parse::new();
        parser.update_content(content).unwrap();

        assert_eq!(
            parser.nodes,
            vec![
                ("asdf".into(), ("This is a sample note".into(), 1, Vec::new())),
                ("ghjk".into(), ("Second note".into(), 7, vec![Link { path: None, note: Some("asdf".into()), text: None }]))
            ].into_iter().collect()
        );

        assert_eq!(
            parser.backlinks,
            vec![
                (Link { path: None, note: Some("asdf".into()), text: None }, vec!["ghjk".into()])
            ].into_iter().collect()
        );
    }
}
