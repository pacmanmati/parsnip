use crate::tag::Tag;

#[derive(Debug)]
pub struct Node {
    pub tag: Tag,
    pub child: ChildrenType,
    // pub(crate) parent: Option<&'a Node<'a>>,
}

#[derive(Debug)]
pub enum ChildrenType {
    None,
    Child(Box<Node>),
    Children(Vec<Node>),
}
