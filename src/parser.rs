use std::collections::HashMap;
use pulldown_cmark::{Parser, Tag, HeadingLevel, Event};

pub type NodeId = String;

fn parse_header(input: &str) -> Option<(String, String)> {
    let parts = input.split("-").collect::<Vec<_>>();
    dbg!(&parts);
    
    match parts[..] {
        [a, b] => Some((a.to_string(), b.to_string())),
        _ => None
    }
}

pub struct Parse {
    nodes: HashMap<NodeId, (String, Vec<NodeId>)>,
    backlinks: HashMap<NodeId, Vec<NodeId>>,
}

impl Parse {
    pub fn new() -> Parse {
        Parse {
            nodes: HashMap::new(),
            backlinks: HashMap::new(),
        }
    }

    pub fn update_content(&mut self, content: &str) -> Result<(), String> {
        let mut source = Parser::new(content).into_offset_iter();

        let mut nodes = HashMap::new();
        let mut backlinks = HashMap::new();

        let mut current_node: Option<(NodeId, (String, Vec<NodeId>))> = None;

        while let Some((elm, range)) = source.next() {
            match elm {
                Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                    dbg!(&range);
                    if let Some((Event::Text(title),_)) = source.next() {
                        if let Some(current_node) = current_node.take() {
                            dbg!(&range);
                            // insert backlinks
                            for link in &current_node.1.1 {
                                backlinks.entry(link.to_string()).or_insert(Vec::new())
                                    .push(current_node.0.to_string());
                            }

                            nodes.insert(current_node.0, current_node.1);

                        }

                        if let Some((id, title)) = parse_header(&title) {
                            current_node = Some((id, (title, Vec::new())));
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
                        elm.1.1.push(link[1..].to_string());
                    }
                },
                _ => {}
            }
        }

        if let Some(current_node) = current_node.take() {
            // insert backlinks
            for link in &current_node.1.1 {
                backlinks.entry(link.to_string()).or_insert(Vec::new())
                    .push(current_node.0.to_string());
                }

            nodes.insert(current_node.0, current_node.1);

        }

        self.nodes = nodes;
        self.backlinks = backlinks;

        dbg!(&self.nodes, &self.backlinks);


        Ok(())
    }

    pub fn go_to(&mut self, cursor: &str) -> Result<String, String> {
        dbg!(&cursor);

        Ok("".into())
    }
}
