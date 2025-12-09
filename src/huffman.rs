use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Leaf { token: u8, freq: u64 },
    Node { left: Box<Tree>, right: Box<Tree> },
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq().cmp(&other.freq())
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Tree {
    pub fn new_leaf(token: u8, freq: u64) -> Self {
        Tree::Leaf { token, freq }
    }

    pub fn new_node(left: Box<Tree>, right: Box<Tree>) -> Self {
        Tree::Node { left, right }
    }

    pub fn token(&self) -> Option<u8> {
        match self {
            Tree::Leaf { token, .. } => Some(*token),
            _ => None,
        }
    }

    pub fn freq(&self) -> u64 {
        match self {
            Tree::Leaf { freq, .. } => *freq,
            Tree::Node { left, right } => left.freq() + right.freq(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreqTable {
    freqs: [u64; 256],
}

impl FreqTable {
    pub fn new() -> Self {
        FreqTable { freqs: [0; 256] }
    }

    pub fn update(&mut self, token: u8, freq: u64) {
        self.freqs[token as usize] += freq;
    }

    pub fn get(&self, token: u8) -> u64 {
        self.freqs[token as usize]
    }
}

pub fn construct_freq_table(data: &[u8]) -> FreqTable {
    let mut table = FreqTable::new();

    for &byte in data {
        table.update(byte, 1);
    }

    table
}

pub fn construct_huffman_tree(freq_table: &FreqTable) -> Tree {
    let mut minheap: BinaryHeap<Reverse<Tree>> = BinaryHeap::new();

    for (token, freq) in freq_table.freqs.iter().enumerate() {
        if *freq > 0 {
            minheap.push(Reverse(Tree::new_leaf(token as u8, *freq)));
        }
    }

    while minheap.len() > 1 {
        let left = minheap.pop().unwrap().0;
        let right = minheap.pop().unwrap().0;

        minheap.push(Reverse(Tree::new_node(Box::new(left), Box::new(right))));
    }

    minheap.pop().unwrap().0
}
