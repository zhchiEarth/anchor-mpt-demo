use anchor_lang::solana_program::keccak::hash;
use primitive_types_solana::H256;
use rlp::{Prototype, Rlp};

use crate::errors::TrieError;
use crate::nibbles::Nibbles;
use crate::node::{empty_children, Node};

pub type TrieResult<T> = Result<T, TrieError>;
const HASHED_LENGTH: usize = 32;

#[derive(Debug)]
pub struct EthTrie {
    root: Node,
    root_hash: H256,
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    // db: HashMap<Vec<u8>, Vec<u8>>,
}

impl EthTrie {
    pub fn new(keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>, root_hash: H256) -> Self {
        Self {
            root: Node::from_hash(root_hash),
            root_hash,
            keys,
            values,
        }
    }

    fn get(&self, key: &[u8]) -> TrieResult<Option<Vec<u8>>> {
        let path = &Nibbles::from_raw(key, true);
        let result = self.get_at(&self.root, path, 0);
        if let Err(TrieError::MissingTrieNode {
            node_hash,
            traversed,
            root_hash,
            err_key: _,
        }) = result
        {
            Err(TrieError::MissingTrieNode {
                node_hash,
                traversed,
                root_hash,
                err_key: Some(key.to_vec()),
            })
        } else {
            result
        }
    }

    pub fn verify_proof(
        root_hash: H256,
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> TrieResult<Option<Vec<u8>>> {
        // let mut proof_db: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        // 不支持 hashMap
        let mut key_list: Vec<Vec<u8>> = Vec::new();
        let mut value_list: Vec<Vec<u8>> = Vec::new();

        for node_encoded in proof.into_iter() {
            let hash: H256 = hash(&node_encoded).to_bytes().into();
            // let hash = hash(&node_encoded);
            if root_hash.eq(&hash) || node_encoded.len() >= HASHED_LENGTH {
                // proof_db.insert(hash.as_bytes().to_vec(), node_encoded);
                key_list.push(hash.as_bytes().to_vec());
                value_list.push(node_encoded);
            }
        }
        let trie = EthTrie::new(key_list, value_list, root_hash);
        trie.get(key).or(Err(TrieError::InvalidProof))
        // Ok(None)
    }

    fn get_at(
        &self,
        source_node: &Node,
        path: &Nibbles,
        path_index: usize,
    ) -> TrieResult<Option<Vec<u8>>> {
        let partial = &path.offset(path_index);
        match source_node {
            Node::Empty => Ok(None),
            Node::Leaf(leaf) => {
                if &leaf.key == partial {
                    Ok(Some(leaf.value.clone()))
                } else {
                    Ok(None)
                }
            }
            Node::Branch(branch) => {
                let borrow_branch = branch;

                if partial.is_empty() || partial.at(0) == 16 {
                    Ok(borrow_branch.value.clone())
                } else {
                    let index = partial.at(0);
                    self.get_at(&borrow_branch.children[index], path, path_index + 1)
                }
            }
            Node::Extension(extension) => {
                let extension = extension;

                let prefix = &extension.prefix;
                let match_len = partial.common_prefix(prefix);
                if match_len == prefix.len() {
                    self.get_at(&extension.node, path, path_index + match_len)
                } else {
                    Ok(None)
                }
            }
            Node::Hash(hash_node) => {
                let node_hash = hash_node.hash;
                let node =
                    self.recover_from_db(node_hash)?
                        .ok_or_else(|| TrieError::MissingTrieNode {
                            node_hash,
                            traversed: Some(path.slice(0, path_index)),
                            root_hash: Some(self.root_hash),
                            err_key: None,
                        })?;
                self.get_at(&node, path, path_index)
            }
        }
    }

    fn decode_node(data: &[u8]) -> TrieResult<Node> {
        let r = Rlp::new(data);

        match r.prototype()? {
            Prototype::Data(0) => Ok(Node::Empty),
            Prototype::List(2) => {
                let key = r.at(0)?.data()?;
                let key = Nibbles::from_compact(key);

                if key.is_leaf() {
                    Ok(Node::from_leaf(key, r.at(1)?.data()?.to_vec()))
                } else {
                    let n = Self::decode_node(r.at(1)?.as_raw())?;

                    Ok(Node::from_extension(key, n))
                }
            }
            Prototype::List(17) => {
                let mut nodes = empty_children();
                #[allow(clippy::needless_range_loop)]
                for i in 0..nodes.len() {
                    let rlp_data = r.at(i)?;
                    let n = Self::decode_node(rlp_data.as_raw())?;
                    nodes[i] = n;
                }

                // The last element is a value node.
                let value_rlp = r.at(16)?;
                let value = if value_rlp.is_empty() {
                    None
                } else {
                    Some(value_rlp.data()?.to_vec())
                };

                Ok(Node::from_branch(nodes, value))
            }
            _ => {
                if r.is_data() && r.size() == HASHED_LENGTH {
                    let hash = H256::from_slice(r.data()?);
                    Ok(Node::from_hash(hash))
                } else {
                    Err(TrieError::InvalidData)
                }
            }
        }
    }

    fn recover_from_db(&self, key: H256) -> TrieResult<Option<Node>> {
        for (i, k) in self.keys.iter().enumerate() {
            if k.eq(key.as_bytes()) {
                // let v = Self::decode_node(&self.values[i])?;
                return Ok(Some(Self::decode_node(&self.values[i])?));
            }
        }

        // let node = match self.db.get(key.as_bytes()) {
        //     Some(value) => Some(Self::decode_node(&value)?),
        //     None => None,
        // };
        Ok(None)
    }
}
