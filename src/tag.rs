use std::collections::BTreeMap;

/// The information relating to a particular tag.
/// For example, an input of `<h1 id="header">Hello world</h1>` would yield the following [Tag]:
/// ```
/// Tag {
///     element: "h1",
///     inner: "hello world"
///     attributes: Some({"id": ["header"]}),
/// }
/// ```
#[derive(Debug)]
pub struct Tag {
    pub element: String,
    pub inner: Option<String>,
    pub attributes: Option<BTreeMap<String, Vec<String>>>,
}
