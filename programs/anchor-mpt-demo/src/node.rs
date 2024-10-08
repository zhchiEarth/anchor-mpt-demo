use std::sync::Arc;

use ethereum_types::H256;

use crate::nibbles::Nibbles;

#[derive(Debug, Clone)]
pub enum Node {
    Empty,
    Leaf(Arc<LeafNode>),
    Extension(Arc<ExtensionNode>),
    Branch(Arc<BranchNode>),
    Hash(Arc<HashNode>),
}

impl Node {
    pub fn from_leaf(key: Nibbles, value: Vec<u8>) -> Self {
        let leaf = Arc::new(LeafNode { key, value });
        Node::Leaf(leaf)
    }

    pub fn from_branch(children: [Node; 16], value: Option<Vec<u8>>) -> Self {
        let branch = Arc::new(BranchNode { children, value });
        Node::Branch(branch)
    }

    pub fn from_extension(prefix: Nibbles, node: Node) -> Self {
        let ext = Arc::new(ExtensionNode { prefix, node });
        Node::Extension(ext)
    }

    pub fn from_hash(hash: H256) -> Self {
        let hash_node = Arc::new(HashNode { hash });
        Node::Hash(hash_node)
    }
}

#[derive(Debug)]
pub struct LeafNode {
    pub key: Nibbles,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub struct BranchNode {
    pub children: [Node; 16],
    pub value: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct ExtensionNode {
    pub prefix: Nibbles,
    pub node: Node,
}

#[derive(Debug)]
pub struct HashNode {
    pub hash: H256,
}

pub fn empty_children() -> [Node; 16] {
    [
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
    ]
}
