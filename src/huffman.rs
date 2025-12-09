use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

use bitvec::vec::BitVec;
use serde::{Deserialize, Serialize};

/// Huffman tree node.
/// Convention: Left => false (0), Right => true (1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tree {
    Leaf {
        token: u8,
        freq: u64,
    },
    Node {
        left: Box<Tree>,
        right: Box<Tree>,
        freq: u64,
    },
}

impl Tree {
    fn new_leaf(token: u8, freq: u64) -> Self {
        Tree::Leaf { token, freq }
    }

    fn new_node(left: Tree, right: Tree) -> Self {
        let freq = left.freq() + right.freq();
        Tree::Node {
            left: Box::new(left),
            right: Box::new(right),
            freq,
        }
    }

    pub fn freq(&self) -> u64 {
        match self {
            Tree::Leaf { freq, .. } => *freq,
            Tree::Node { freq, .. } => *freq,
        }
    }
}

/// Used to get deterministic ordering in the heap.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HeapNode {
    freq: u64,
    id: usize,
    tree: Tree,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Order by freq, then id.
        self.freq
            .cmp(&other.freq)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FreqsTable {
    freqs: [u64; 256],
}

impl FreqsTable {
    pub fn new() -> Self {
        FreqsTable { freqs: [0; 256] }
    }

    pub fn add(&mut self, token: u8, freq: u64) {
        self.freqs[token as usize] = self.freqs[token as usize].saturating_add(freq);
    }
}

/// Encoder table: for each byte value store its bit code.
#[derive(Debug, Clone, PartialEq, Eq)]
struct EncoderTable {
    encoder: [BitVec; 256],
}

impl EncoderTable {
    pub fn new() -> Self {
        EncoderTable {
            encoder: std::array::from_fn(|_| BitVec::new()),
        }
    }

    pub fn get(&self, token: u8) -> &BitVec {
        &self.encoder[token as usize]
    }

    fn set(&mut self, token: u8, code: BitVec) {
        self.encoder[token as usize] = code;
    }
}

/// Encoded message: contains the Huffman tree, the encoded bits and the
/// original length of the input (required to properly decode edge-cases).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub tree: Tree,
    pub encoded_data: BitVec,
    pub original_len: usize,
}

/// Public API
pub struct Huffman;

impl Huffman {
    /// Encode a byte slice into a `Message`.
    ///
    /// Returns an error string on failure.
    /// Returns an empty message if the input is empty.
    pub fn encode(bytes: &[u8]) -> Result<Message, String> {
        if bytes.is_empty() {
            return Ok(Message {
                tree: Tree::new_leaf(0, 0),
                encoded_data: BitVec::new(),
                original_len: 0,
            });
        }

        let freq_table = construct_freqs_table(bytes);
        let tree = construct_huffman_tree(&freq_table)?;
        let encoder = construct_encoder_table(&tree);

        let mut encoded_data = BitVec::new();

        for &b in bytes {
            let code = encoder.get(b);
            encoded_data.extend(code.iter().by_vals());
        }

        Ok(Message {
            tree,
            encoded_data,
            original_len: bytes.len(),
        })
    }

    /// Decode a `Message` back into the original bytes.
    pub fn decode(message: &Message) -> Result<Vec<u8>, String> {
        if message.original_len == 0 {
            return Ok(Vec::new());
        }

        // Special-case: if the tree is a single leaf, then the encoding
        // uses a non-empty code per symbol.
        if let Tree::Leaf { token, .. } = &message.tree {
            return Ok(vec![*token; message.original_len]);
        }

        let mut decoded = Vec::with_capacity(message.original_len);
        let mut node = &message.tree;

        for bit in message.encoded_data.iter() {
            match node {
                &Tree::Node {
                    ref left,
                    ref right,
                    ..
                } => {
                    let next: &Tree = if *bit { right.as_ref() } else { left.as_ref() };
                    node = next;

                    if let &Tree::Leaf { token, .. } = node {
                        decoded.push(token);
                        node = &message.tree;
                        if decoded.len() == message.original_len {
                            break;
                        }
                    }
                }
                &Tree::Leaf { token, .. } => {
                    // This could only happen for degenerate trees.
                    // If so, push the token and reset the node.
                    decoded.push(token);
                    node = &message.tree;
                    if decoded.len() == message.original_len {
                        break;
                    }
                }
            }
        }

        // If traversal ended exactly on a leaf without another bit to trigger
        // pushing it during the loop.
        if decoded.len() < message.original_len {
            if let Tree::Leaf { token, .. } = node {
                decoded.push(*token);
            }
        }

        if decoded.len() != message.original_len {
            return Err(format!(
                "Decoded length mismatch: expected {}, got {}",
                message.original_len,
                decoded.len()
            ));
        }

        Ok(decoded)
    }
}

/// Build frequency table from input bytes.
fn construct_freqs_table(data: &[u8]) -> FreqsTable {
    let mut freqs = FreqsTable::new();
    for &b in data {
        freqs.add(b, 1);
    }
    freqs
}

/// Construct Huffman tree from frequency table.
/// Returns Error if there are no symbols.
fn construct_huffman_tree(freq_table: &FreqsTable) -> Result<Tree, String> {
    // Build a minheap using Reverse. HeapNode::cmp orders by freq then id.
    let mut minheap: BinaryHeap<std::cmp::Reverse<HeapNode>> = BinaryHeap::new();
    let mut next_id: usize = 0;

    for (token, &freq) in freq_table.freqs.iter().enumerate() {
        if freq > 0 {
            let node = HeapNode {
                freq,
                id: next_id,
                tree: Tree::new_leaf(token as u8, freq),
            };
            next_id = next_id.saturating_add(1);
            minheap.push(std::cmp::Reverse(node));
        }
    }

    if minheap.is_empty() {
        return Err("Empty frequency table: cannot construct Huffman tree".to_string());
    }

    // If there's only one symbol, return the single leaf. We'll ensure in the encoder
    // that it gets assigned a non-empty code.
    while minheap.len() > 1 {
        let Reverse(left_node) = minheap.pop().unwrap();
        let Reverse(right_node) = minheap.pop().unwrap();

        let combined_tree = Tree::new_node(left_node.tree, right_node.tree);
        let combined_freq = left_node.freq + right_node.freq;

        let new_node = HeapNode {
            freq: combined_freq,
            id: next_id,
            tree: combined_tree,
        };
        next_id = next_id.saturating_add(1);
        minheap.push(std::cmp::Reverse(new_node));
    }

    let root = minheap.pop().unwrap().0.tree;
    Ok(root)
}

/// Build encoder table from the Huffman tree.
///
/// Special handling:
/// - If the tree consists of a single leaf, assign a non-empty code (single 0 bit)
///   to that symbol so that encoding produces bits to represent repeated occurrences.
fn construct_encoder_table(tree: &Tree) -> EncoderTable {
    let mut encoder = EncoderTable::new();

    fn traverse(node: &Tree, code: &mut BitVec, table: &mut EncoderTable) {
        match node {
            Tree::Leaf { token, .. } => {
                table.set(*token, code.clone());
            }
            Tree::Node { left, right, .. } => {
                code.push(false);
                traverse(left, code, table);
                code.pop();

                code.push(true);
                traverse(right, code, table);
                code.pop();
            }
        }
    }

    if let Tree::Leaf { token, .. } = tree {
        let mut code = BitVec::new();
        code.push(false);
        encoder.set(*token, code);
        return encoder;
    }

    let mut code = BitVec::new();
    traverse(tree, &mut code, &mut encoder);
    encoder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_compression() -> Result<(), String> {
        let text = "Hello World! Hello Huffman!";
        let bytes = text.as_bytes();

        let message = Huffman::encode(bytes)?;
        let decoded_bytes = Huffman::decode(&message)?;
        let decoded = String::from_utf8(decoded_bytes).map_err(|e| e.to_string())?;

        assert_eq!(text, decoded);

        Ok(())
    }

    #[test]
    fn test_empty_compression() -> Result<(), String> {
        let text = "";
        let bytes = text.as_bytes();

        let message = Huffman::encode(bytes)?;
        let decoded_bytes = Huffman::decode(&message)?;
        let decoded = String::from_utf8(decoded_bytes).map_err(|e| e.to_string())?;

        assert_eq!(text, decoded);

        Ok(())
    }
}
