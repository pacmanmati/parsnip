use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    error::Error,
    fmt::Display,
};

use crate::{
    lex::Token,
    node::{ChildrenType, Node},
    tag::Tag,
};

#[derive(Debug)]
pub(crate) enum ParserError {
    MalformedHTML,
    InternalError(String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: match on self to determine error
        write!(f, "ParserError: {:?}", self)
    }
}

impl Error for ParserError {}

pub fn parse(tokens: Vec<Token>) -> Result<Option<Node>, Box<dyn Error>> {
    make_tree(tokens)
}

fn make_tree(tokens: Vec<Token>) -> Result<Option<Node>, Box<dyn Error>> {
    // we want to construct a queue that allows us to walk backwards up the tree,
    // constructing nodes and slotting the previous node in
    //  div 0
    //      div 1
    //          h1 2
    //          h2 3
    //          span 4
    //          ul 5
    //              li 6
    //              li 7
    //      div 8
    //          p 9
    //          div 10
    //              p 11
    //      div 12
    //          a 13
    //
    // algorithm:
    // make a queue like so: 0 1 8 12 2 3 4 5 9 10 13 6 7 11
    // the idea is that we start from the back and create the tag for that index then pop it.
    // we store that tag against it's index in a hashmap
    // when we encounter 10 for example, we calculate its dependencies (look at start and end, find all starts)
    // then look those up in the table (they should already exist)
    // this way we can avoid recursion

    let mut nodes: HashMap<usize, Option<Node>> = HashMap::new();

    // create the queue
    let mut queue = breadth_first_queue(&tokens, 0, None)?;

    while let Some((begin, end)) = queue.pop_back() {
        let mut children = breadth_first_queue(&tokens, begin, Some(1))?
            .iter()
            .skip(1)
            .map(|(begin, _)| nodes.get_mut(begin).unwrap().take())
            .collect::<Vec<_>>();
        let child = match children.len() {
            0 => ChildrenType::None,
            1 => ChildrenType::Child(Box::new(children.first_mut().unwrap().take().unwrap())),
            _ => ChildrenType::Children(children.into_iter().map(|node| node.unwrap()).collect()),
        };
        // get this one's inner
        let inner = if end > begin {
            match &tokens.get(begin + 1).unwrap() {
                Token::Inner(inner) => Some(inner.clone()),
                _ => None, // this should probably be an error
            }
        } else {
            None
        };
        let node = make_node(&tokens, begin, child, inner)?;
        nodes.insert(begin, Some(node));
    }
    Ok(nodes.get_mut(&0).unwrap().take())
}

fn breadth_first_queue(
    tokens: &Vec<Token>,
    start: usize,
    max_depth: Option<u32>,
) -> Result<VecDeque<(usize, usize)>, ParserError> {
    let mut queue = VecDeque::new();
    let mut todo = VecDeque::new();
    let mut depth = 0_u32;
    let mut frontier = 1_u32;
    let mut next_frontier = 0_u32;

    todo.push_back(start);

    queue.push_back((start, find_closing(tokens.clone(), start)?.unwrap()));
    let t = tokens.clone();

    while let Some(current) = todo.pop_back() {
        if frontier == 0 {
            frontier = next_frontier;
            next_frontier = 0;
            depth += 1;
        }
        frontier -= 1;
        if let Some(max_depth) = max_depth {
            if max_depth <= depth {
                break;
            }
        }
        let mut iter = t.iter().enumerate().skip(current + 1);
        let closing = find_closing(tokens.clone(), current)?.unwrap();
        let len = tokens.len();
        (closing..len).for_each(|_| {
            iter.next_back();
        });
        while let Some((child, token)) = iter.next() {
            assert!(child != 0);
            match token {
                Token::Close(_) => {
                    continue;
                } // this one we shouldn't encounter
                Token::Inner(_) => continue, // this one we should skip
                _ => {}                      // these are okay
            };

            todo.push_back(child);
            next_frontier += 1;
            match token {
                Token::Open(_, _) => {
                    let closing = find_closing(tokens.clone(), child)?.unwrap();
                    // skip past the end tag
                    (child..closing).for_each(|_| {
                        iter.next();
                    });
                    queue.push_back((child, closing));
                }
                Token::SelfClose(_, _) => {
                    queue.push_back((child, child));
                }
                _ => {
                    return Err(ParserError::MalformedHTML);
                }
            };
        }
    }
    Ok(queue)
}

fn find_closing(tokens: Vec<Token>, id: usize) -> Result<Option<usize>, ParserError> {
    if let Token::SelfClose(_, _) = tokens.get(id).unwrap() {
        return Ok(Some(id));
    }
    if let Token::Open(open, _) = tokens.get(id).unwrap() {
        let mut nested = 0;
        for (idx, token) in tokens.iter().enumerate().skip(id + 1) {
            match token {
                Token::Open(tag, _) => {
                    if tag == open {
                        nested += 1
                    }
                }
                Token::Close(tag) => {
                    if tag == open {
                        if nested == 0 {
                            // found the closing tag
                            return Ok(Some(idx));
                        } else {
                            nested -= 1
                        }
                    }
                }
                _ => {}
            }
        }

        return Ok(None);
    }
    Err(ParserError::InternalError(format!(
        "Expected an opening token at {id}."
    )))
}

fn make_node(
    tokens: &[Token],
    id: usize,
    child: ChildrenType,
    inner: Option<String>,
) -> Result<Node, ParserError> {
    let tag = make_tag(tokens, id, inner)?;
    Ok(Node { tag, child })
}

fn make_tag(tokens: &[Token], id: usize, inner: Option<String>) -> Result<Tag, ParserError> {
    let token = tokens.get(id).unwrap();
    match token {
        Token::Open(tag, rest) => Ok(Tag {
            element: tag.clone(),
            inner,
            attributes: extract_attributes(rest.clone()),
        }),
        Token::SelfClose(tag, rest) => Ok(Tag {
            element: tag.clone(),
            inner,
            attributes: extract_attributes(rest.clone()),
        }),
        _ => Err(ParserError::InternalError("Something went wrong.".into())),
    }
}

fn extract_attributes(line: String) -> Option<BTreeMap<String, Vec<String>>> {
    // <img class="image square" src="images/dog.png">
    if line.is_empty() {
        None
    } else {
        let mut map = BTreeMap::new();
        // let mut iter = line.split('=').into_iter();
        // while let Some(current) = iter.next() {
        //     iter.next();
        // }
        // let mut inside_quote = false;
        // let mut key = String::new();
        // let mut value = String::new();
        // let mut values = Vec::new();

        // for c in line.chars() {
        //     if c == ' ' {
        //         if inside_quote {
        //             values.push(std::mem::take(&mut value));
        //         } else {
        //             map.insert(std::mem::take(&mut key), std::mem::take(&mut values));
        //         }
        //     } else if c == '\'' || c == '"' {
        //         inside_quote = !inside_quote;
        //     } else {
        //         value.push(c);
        //     }
        // }
        Some(map)
    }
}

enum AttributeToken {
    Equals,
    Key(String),
    Value(String),
}

fn attrib_split(line: String) -> Vec<AttributeToken> {
    // let mut
    // for c in line.chars() {
    //     if c == '=' {
    //     } else if c == ' ' {
    //     }
    // }
    vec![]
}
