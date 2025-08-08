use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, RwLock, RwLockReadGuard},
};

/// Represents a single-threaded context for DOM nodes.
///
/// Will use `Rc<RefCell<T>>` for non-thread-safe access to nodes.
#[derive(Debug, Clone)]
pub struct SingleThreaded;

/// Represents a multi-threaded context for DOM nodes.
///
/// Will use `Arc<RwLock<T>>` for thread-safe access to nodes.
#[derive(Debug, Clone)]
pub struct MultiThreaded;

/// A trait that defines the context for how DOM nodes are managed.
pub trait NodeContext {
    /// The type of a single node in the DOM tree.
    type Node<T: Clone>: Clone;

    /// The type of children nodes in the DOM tree.
    type Children<T: Clone>: Clone + IntoIterator<Item = Self::Node<T>>;

    /// Should return a new node of type `T` wrapped in the appropriate context.
    fn new_node<T: Clone>(node: &T) -> Self::Node<T>;

    /// Should return an empty collection of children nodes.
    fn empty_children<T: Clone>() -> Self::Children<T>;
}

impl NodeContext for SingleThreaded {
    type Node<T: Clone> = Rc<RefCell<T>>;
    type Children<T: Clone> = Vec<Self::Node<T>>;

    fn new_node<T: Clone>(node: &T) -> Self::Node<T> {
        Rc::new(RefCell::new(node.clone()))
    }

    fn empty_children<T: Clone>() -> Self::Children<T> {
        Vec::new()
    }
}

impl NodeContext for MultiThreaded {
    type Node<T: Clone> = Arc<RwLock<T>>;
    type Children<T: Clone> = Vec<Self::Node<T>>;

    fn new_node<T: Clone>(node: &T) -> Self::Node<T> {
        Arc::new(RwLock::new(node.clone()))
    }

    fn empty_children<T: Clone>() -> Self::Children<T> {
        Vec::new()
    }
}

/// Represents an HTML element in the DOM tree.
///
/// # Type Parameters
/// * `Context` - The threading model used for the element, which can be either `SingleThreaded` or `MultiThreaded`.
///
/// # Fields
/// * `id` - A unique identifier for the element.
/// * `attributes` - A map of attributes associated with the element, where keys are attribute names and values are attribute values.
/// * `tag_name` - The name of the HTML tag for the element (e.g., "div", "span").
/// * `children` - The child nodes of the element, which are represented as a collection of nodes in the specified context.
#[derive(Clone)]
pub struct Element<Context: NodeContext + Clone> {
    pub id: u16,
    pub attributes: HashMap<String, String>,
    pub tag_name: String,
    pub children: Context::Children<DocumentNode<Context>>,
}

impl<Context: NodeContext + Clone> Default for Element<Context> {
    fn default() -> Self {
        Element {
            id: 0,
            attributes: HashMap::new(),
            tag_name: String::new(),
            children: Context::empty_children(),
        }
    }
}

impl<Context: NodeContext + Clone> PartialEq for Element<Context> {
    /// Compares two elements for equality based on their unique identifier.
    /// Does not compare other fields like tag name or attributes or children.
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Represents a node in the DOM tree, which can be an element or text node.
///
/// # Type Parameters
/// * `Context` - The threading model used for the node, which can be either `SingleThreaded` or `MultiThreaded`.
///
/// # Variants
/// * `Element` - Represents an HTML element with a tag name, attributes, and children.
/// * `Text` - Represents a text node containing text content.
#[derive(Clone, PartialEq)]
pub enum DocumentNode<Context: NodeContext + Clone> {
    /// Represents different types of nodes in the DOM tree.
    /// e.g. <html>, <body>, <div>, etc.
    Element(Element<Context>),

    /// Represents a text node containing text content.
    /// e.g. "Hello, World!".
    /// This is always a leaf node in the DOM tree.
    ///
    /// It does **NOT** have children.
    Text(String),
}

impl<Context: NodeContext + Clone + Debug> DocumentNode<Context> {
    pub fn as_element(&self) -> Option<&Element<Context>> {
        if let DocumentNode::Element(element) = self {
            Some(element)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&String> {
        if let DocumentNode::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
}

impl From<DocumentNode<SingleThreaded>> for DocumentNode<MultiThreaded>
where
    DocumentNode<SingleThreaded>: Clone,
{
    /// Converts a single-threaded document node into a threaded-safe document node.
    ///
    /// Should be called before rendering, and after parsing.
    fn from(node: DocumentNode<SingleThreaded>) -> Self {
        match node {
            DocumentNode::Element(element) => convert_element(element),
            DocumentNode::Text(text) => DocumentNode::Text(text),
        }
    }
}

/// Converts a single-threaded element into a multi-threaded element.
fn convert_element(element: Element<SingleThreaded>) -> DocumentNode<MultiThreaded> {
    DocumentNode::Element(Element {
        id: element.id,
        tag_name: element.tag_name,
        attributes: element.attributes,
        children: element.children.into_iter().map(convert_node).collect(),
    })
}

/// Converts a single-threaded node into a multi-threaded node.
fn convert_node(
    child: Rc<RefCell<DocumentNode<SingleThreaded>>>,
) -> Arc<RwLock<DocumentNode<MultiThreaded>>> {
    Arc::new(RwLock::new(DocumentNode::from((*child.borrow()).clone())))
}

/// Converts a single-threaded node into a multi-threaded node using a shared conversion map.
fn convert_node_shared(
    child: &Rc<RefCell<DocumentNode<SingleThreaded>>>,
    conversion_map: &mut HashMap<
        *const RefCell<DocumentNode<SingleThreaded>>,
        Arc<RwLock<DocumentNode<MultiThreaded>>>,
    >,
) -> Arc<RwLock<DocumentNode<MultiThreaded>>> {
    let ptr = Rc::as_ptr(child);

    // Check if we've already converted this node
    if let Some(existing) = conversion_map.get(&ptr) {
        return existing.clone();
    }

    match &*child.borrow() {
        DocumentNode::Element(element) => {
            // First create the Arc for this node (without children) to avoid cycles
            let new_node = Arc::new(RwLock::new(DocumentNode::Element(Element {
                id: element.id,
                tag_name: element.tag_name.clone(),
                attributes: element.attributes.clone(),
                children: Vec::new(), // Temporary empty children
            })));

            // Store in map before recursing to handle potential cycles
            conversion_map.insert(ptr, new_node.clone());

            // Now convert children
            let converted_children: Vec<Arc<RwLock<DocumentNode<MultiThreaded>>>> = element
                .children
                .iter()
                .map(|child| convert_node_shared(child, conversion_map))
                .collect();

            // Update the children
            if let Ok(mut node_guard) = new_node.write() {
                if let DocumentNode::Element(elem) = &mut *node_guard {
                    elem.children = converted_children;
                }
            }

            new_node
        }
        DocumentNode::Text(text) => {
            let new_node = Arc::new(RwLock::new(DocumentNode::Text(text.clone())));
            conversion_map.insert(ptr, new_node.clone());
            new_node
        }
    }
}

pub struct DomIndex<Context: NodeContext + Debug + Clone> {
    pub flat: Vec<Context::Node<DocumentNode<Context>>>,
    pub id: HashMap<u16, Context::Node<DocumentNode<Context>>>,
    pub tag: HashMap<String, Vec<Context::Node<DocumentNode<Context>>>>,
}

impl DomIndex<MultiThreaded> {
    /// Returns a guard that can be used to access the element directly.
    pub fn first_element_by_tag(
        &self,
        tag_name: &str,
    ) -> Option<RwLockReadGuard<DocumentNode<MultiThreaded>>> {
        self.tag.get(tag_name)?.first()?.read().ok()
    }

    /// Gets all elements by tag name, returning a vector of guards.
    pub fn all_elements_by_tag(
        &self,
        tag_name: &str,
    ) -> Vec<RwLockReadGuard<DocumentNode<MultiThreaded>>> {
        self.tag
            .get(tag_name)
            .map(|nodes| nodes.iter().filter_map(|node| node.read().ok()).collect())
            .unwrap_or_default()
    }
}

impl<Context: NodeContext + Debug + Clone> Default for DomIndex<Context> {
    fn default() -> Self {
        DomIndex {
            flat: Vec::new(),
            id: HashMap::new(),
            tag: HashMap::new(),
        }
    }
}

/// Represents the root of a document, which is a collection of nodes in the specified context.
pub struct DocumentRoot<Context: NodeContext + Debug + Clone> {
    pub nodes: Vec<Context::Node<DocumentNode<Context>>>,
    pub index: DomIndex<Context>,
}

impl<Context: NodeContext + Debug + Clone> Default for DocumentRoot<Context> {
    fn default() -> Self {
        DocumentRoot {
            nodes: Vec::new(),
            index: DomIndex::default(),
        }
    }
}

impl<Context: NodeContext + Debug + Clone> DocumentRoot<Context> {
    /// Creates a new `DocumentRoot` with an empty vector of nodes.
    pub fn new(nodes: Vec<Context::Node<DocumentNode<Context>>>, index: DomIndex<Context>) -> Self {
        DocumentRoot { nodes, index }
    }

    /// Converts a single-threaded DocumentRoot to a multi-threaded DocumentRoot with shared references.
    pub fn convert(
        &mut self,
        single_threaded_root: DocumentRoot<SingleThreaded>,
    ) -> DocumentRoot<MultiThreaded> {
        let mut conversion_map: HashMap<
            *const RefCell<DocumentNode<SingleThreaded>>,
            Arc<RwLock<DocumentNode<MultiThreaded>>>,
        > = HashMap::new();

        DocumentRoot {
            nodes: single_threaded_root
                .nodes
                .iter()
                .map(|node| convert_node_shared(node, &mut conversion_map))
                .collect(),
            index: {
                DomIndex {
                    flat: single_threaded_root
                        .index
                        .flat
                        .iter()
                        .map(|node| convert_node_shared(node, &mut conversion_map))
                        .collect(),
                    id: single_threaded_root
                        .index
                        .id
                        .iter()
                        .map(|(k, v)| (*k, convert_node_shared(v, &mut conversion_map)))
                        .collect(),
                    tag: single_threaded_root
                        .index
                        .tag
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.iter()
                                    .map(|node| convert_node_shared(node, &mut conversion_map))
                                    .collect(),
                            )
                        })
                        .collect(),
                }
            },
        }
    }
}
