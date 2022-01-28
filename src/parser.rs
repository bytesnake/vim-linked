use std::collections::HashMap;
use pulldown_cmark::{Parser, Tag, HeadingLevel, Event};
use tinyjson::JsonValue;

pub type NodeId = String;

fn parse_header(input: &str) -> Option<(String, String)> {
    let parts = input.split("-").collect::<Vec<_>>();
    
    match parts[..] {
        [a, b] => Some((a.trim().to_string(), b.trim().to_string())),
        _ => None
    }
}

pub struct Parse {
    nodes: HashMap<NodeId, (String, usize, Vec<NodeId>)>,
    backlinks: HashMap<NodeId, Vec<NodeId>>,
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

    pub fn update_content(&mut self, content: &str) -> Result<(), String> {
        let mut source = Parser::new(content).into_offset_iter();

        let mut nodes = HashMap::new();
        let mut backlinks = HashMap::new();

        let mut current_node: Option<(NodeId, (String, usize, Vec<NodeId>))> = None;

        while let Some((elm, range)) = source.next() {
            match elm {
                Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                    if let Some((Event::Text(title),_)) = source.next() {
                        if let Some(current_node) = current_node.take() {
                            // insert backlinks
                            for link in &current_node.1.2 {
                                backlinks.entry(link.to_string()).or_insert(Vec::new())
                                    .push(current_node.0.to_string());
                            }

                            nodes.insert(current_node.0, current_node.1);

                        }

                        let line_num = content[..range.start].matches("\n").count();

                        if let Some((id, title)) = parse_header(&title) {
                            current_node = Some((id, (title, line_num, Vec::new())));
                        } else {
                            current_node = None;
                        }
                    }
                },
                Event::Start(Tag::Link(_, link, _)) => {
                    if !link.starts_with("@") {
                        continue;
                    }

                    if let Some(ref mut elm) = current_node {
                        elm.1.2.push(link[1..].to_string());
                    }
                },
                _ => {}
            }
        }

        if let Some(current_node) = current_node.take() {
            // insert backlinks
            for link in &current_node.1.2 {
                backlinks.entry(link.to_string()).or_insert(Vec::new())
                    .push(current_node.0.to_string());
                }

            nodes.insert(current_node.0, current_node.1);

        }

        self.nodes = nodes;
        self.backlinks = backlinks;
        self.content = content.lines().map(|x| x.to_string()).collect();

        Ok(())
    }

    pub fn go_to(&mut self, cursor: &str) -> Result<String, String> {
        let cursor: JsonValue = cursor.parse().unwrap();
        let line: &f64 = cursor["cursor"][1].get().unwrap();
        let shift: &f64 = cursor["cursor"][2].get().unwrap();
        let mode: String = cursor["mode"].clone().try_into().unwrap();

        // first check if we have a link
        let mut current_id = None;
        let line: &str = &self.content[*line as usize - 1];
        for (elm, range) in Parser::new(line).into_offset_iter() {
            match elm {
                Event::Start(Tag::Link(_, link, _)) => {
                    if !range.contains(&(*shift as usize)) {
                        continue;
                    }

                    current_id = Some(link[1..].to_string());
                    break;
                },
                _ => {}
            }
        }

        match (mode.as_ref(), current_id) {
            ("fort", Some(id)) => {
                match self.nodes.get(&id) {
                    Some(ref node) => Ok(format!("{{ \"line\": {}}}", node.1 + 1)),
                    None => Err(format!("id {} not found", id)),
                }
            },
            ("fort", None) => {
                return Ok("".into());
            },
            _ => panic!("call mode {} not supported!", mode)
        }
    }
}
