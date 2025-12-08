use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use serde::{Deserialize, Serialize};

use bitvec::vec::BitVec;

#[derive(Debug, Clone)]
pub enum Tree<T> {
    Leaf {
        freq: u64,
        token: T,
    },
    Node {
        freq: u64,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T> Tree<T> {
    pub fn freq(&self) -> u64 {
        match self {
            Tree::Leaf { freq, .. } => *freq,
            Tree::Node { freq, .. } => *freq,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedMessage<T: Hash + Eq> {
    freqs: HashMap<T, u64>,
    message: BitVec,
}

impl<T> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.freq() == other.freq()
    }
}

impl<T> Eq for Tree<T> {}

impl<T> Ord for Tree<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq().cmp(&other.freq())
    }
}

impl<T> PartialOrd for Tree<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Construct frequency map from a sequence of tokens.
pub fn construct_freqs<T: Clone + Eq + Hash>(tokens: &Vec<T>) -> HashMap<T, u64> {
    tokens.iter().fold(HashMap::new(), |mut acc, token| {
        *acc.entry(token.clone()).or_insert(0) += 1;
        acc
    })
}

/// Construct Huffman tree from a frequency map.
pub fn construct_tree<T: Clone + Eq>(freqs: &HashMap<T, u64>) -> Tree<T> {
    let mut heap: BinaryHeap<Reverse<Tree<T>>> = BinaryHeap::new();

    for (token, freq) in freqs {
        heap.push(Reverse(Tree::Leaf {
            freq: *freq,
            token: token.clone(),
        }));
    }

    if heap.is_empty() {
        panic!("Cannot construct a Huffman tree from empty frequencies");
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap().0;
        let right = heap.pop().unwrap().0;

        let parent = Tree::Node {
            freq: left.freq() + right.freq(),
            left: Box::new(left),
            right: Box::new(right),
        };

        heap.push(Reverse(parent));
    }

    heap.pop().unwrap().0
}

/// Construct encoder map: token -> bit sequence.
pub fn construct_encoder<T: Clone + Eq + Hash>(tree: &Tree<T>) -> HashMap<T, BitVec> {
    let mut stack: Vec<(&Tree<T>, BitVec)> = vec![(tree, BitVec::new())];
    let mut encoder = HashMap::new();

    while let Some((subtree, code)) = stack.pop() {
        match subtree {
            Tree::Leaf { token, .. } => {
                encoder.insert(token.clone(), code);
            }
            Tree::Node { left, right, .. } => {
                let mut code_left = code.clone();
                let mut code_right = code.clone();

                // convention: left = 0, right = 1
                code_left.push(false);
                code_right.push(true);

                stack.push((left, code_left));
                stack.push((right, code_right));
            }
        }
    }

    encoder
}

pub fn encode_bits<T: Eq + Hash>(encoder: &HashMap<T, BitVec>, tokens: &Vec<T>) -> BitVec {
    tokens.iter().fold(BitVec::new(), |mut acc, token| {
        let bits = encoder.get(token).expect("token missing from encoder map");
        acc.extend_from_bitslice(bits.as_bitslice());
        acc
    })
}

pub fn decode_bits<T: Clone + Eq + Hash>(
    tree: &Tree<T>,
    encoded_message: &BitVec,
    expected_count: usize,
) -> Vec<T> {
    let mut output = Vec::with_capacity(expected_count);
    let mut i = 0usize;

    if let Tree::Leaf { token, .. } = tree {
        for _ in 0..expected_count {
            output.push(token.clone());
        }
        return output;
    }

    let mut current_node = tree;
    while output.len() < expected_count {
        match current_node {
            Tree::Leaf { token, .. } => {
                output.push(token.clone());
                current_node = tree;
            }
            Tree::Node { left, right, .. } => {
                if i >= encoded_message.len() {
                    break;
                }
                if encoded_message[i] {
                    current_node = right.as_ref();
                } else {
                    current_node = left.as_ref();
                }
                i += 1;
            }
        }
    }

    output
}

/// High-level encode function that builds the frequency map and includes it
/// in the serialized output so the decoder can reconstruct the tree.
pub fn encode<'a, T, TokenExtractor>(text: &'a String, extract_tokens: TokenExtractor) -> Vec<u8>
where
    T: Clone + Eq + Hash + Serialize,
    TokenExtractor: Fn(&'a str) -> Vec<T>,
{
    let tokens = extract_tokens(&text);
    let freqs = construct_freqs(&tokens);
    let tree = construct_tree(&freqs);
    let encoder = construct_encoder(&tree);

    let bits = encode_bits(&encoder, &tokens);
    let encoded_message = EncodedMessage {
        freqs: freqs.clone(),
        message: bits,
    };

    rmp_serde::encode::to_vec(&encoded_message).expect("serialization failed")
}

pub fn encode_message<'a, T, TokenExtractor>(
    text: &'a String,
    extract_tokens: TokenExtractor,
    freqs: &HashMap<T, u64>,
) -> Vec<u8>
where
    T: Clone + Eq + Hash + Serialize,
    TokenExtractor: Fn(&'a str) -> Vec<T>,
{
    let tokens = extract_tokens(&text);
    let tree = construct_tree(&freqs);
    let encoder = construct_encoder(&tree);

    let bits = encode_bits(&encoder, &tokens);

    rmp_serde::encode::to_vec(&bits).expect("serialization failed")
}

/// Decode an encoded message that contains its frequency map.
pub fn decode_message<'a, T, TokensToString>(
    message: &'a Vec<u8>,
    tokens_to_string: TokensToString,
) -> String
where
    T: Clone + Eq + Hash + Deserialize<'a>,
    TokensToString: Fn(Vec<T>) -> String,
{
    let EncodedMessage {
        freqs,
        message: bits,
    }: EncodedMessage<T> = rmp_serde::decode::from_slice(message).expect("deserialization failed");

    let expected = freqs.values().sum::<u64>() as usize;
    let tree = construct_tree(&freqs);
    tokens_to_string(decode_bits(&tree, &bits, expected))
}

pub fn decode_with_freqs<'a, T, TokensToString>(
    chars: &'a Vec<u8>,
    tokens_to_string: TokensToString,
    freqs: &HashMap<T, u64>,
) -> String
where
    T: Clone + Eq + Hash + Deserialize<'a>,
    TokensToString: Fn(Vec<T>) -> String,
{
    let bits: BitVec = rmp_serde::decode::from_slice(chars).expect("deserialization failed");

    let expected = freqs.values().sum::<u64>() as usize;
    let tree = construct_tree(&freqs);
    tokens_to_string(decode_bits(&tree, &bits, expected))
}

pub fn char_tokenizer(text: &str) -> Vec<char> {
    text.chars().collect()
}

pub fn chars_to_string(chars: Vec<char>) -> String {
    chars.into_iter().collect()
}

pub fn encode_chars(text: &String) -> Vec<u8> {
    encode::<char, _>(text, |s| char_tokenizer(s))
}

pub fn encode_chars_with_freqs(text: &String, freqs: &HashMap<char, u64>) -> Vec<u8> {
    encode_message::<char, _>(text, |s| char_tokenizer(s), freqs)
}

pub fn decode_chars(chars: &Vec<u8>) -> String {
    decode_message::<char, _>(chars, |tokens| chars_to_string(tokens))
}

pub fn decode_chars_with_freqs(chars: &Vec<u8>, freqs: &HashMap<char, u64>) -> String {
    decode_with_freqs::<char, _>(chars, |tokens| chars_to_string(tokens), freqs)
}
