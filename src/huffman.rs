use std::cmp::Reverse;

// TODO: use a different hasher maybe
use std::collections::{BinaryHeap, HashMap};

use mem_dbg::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, MemSize, MemDbg)]
pub(crate) struct HuffmanNode {
    pub(crate) count: usize,
    pub(crate) symbol: Option<u8>,
    pub(crate) index: usize,
    pub(crate) parent: Option<usize>,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, MemSize, MemDbg)]
pub(crate) struct HuffmanTree {
    pub(crate) root: usize,
    pub(crate) tree: Vec<HuffmanNode>,
    pub(crate) symbols: HashMap<u8, usize>,
}

impl HuffmanTree {
    pub(crate) fn new(input: &[u8]) -> HuffmanTree {
        let mut symbols: HashMap<u8, usize> = HashMap::new();

        for c in input {
            symbols.entry(*c).and_modify(|n| *n += 1).or_insert(1);
        }

        let mut tree: Vec<HuffmanNode> =
            Vec::from_iter(symbols.iter().map(|(symbol, count)| HuffmanNode {
                count: *count,
                symbol: Some(*symbol),
                ..Default::default()
            }));

        // TODO is this a good idea? idk... which way should we really sort?
        tree.sort_unstable();

        symbols.clear();
        let mut heap = BinaryHeap::with_capacity(tree.len());
        for (i, hn) in tree.iter_mut().enumerate() {
            hn.index = i;
            symbols.insert(hn.symbol.unwrap(), i);
            heap.push(Reverse(hn.clone()));
        }

        while heap.len() > 1 {
            let n1 = heap.pop().unwrap().0;
            let n2 = heap.pop().unwrap().0;

            let mut p = HuffmanNode::default();

            p.count = n1.count + n2.count;
            p.left = Some(n1.index);
            p.right = Some(n2.index);
            p.index = tree.len();

            tree[n1.index].parent = Some(p.index);
            tree[n2.index].parent = Some(p.index);

            heap.push(Reverse(p.clone()));
            tree.push(p);
        }

        let root = heap.pop().unwrap().0;
        tree.shrink_to_fit();

        // for (i, t) in tree.iter().enumerate() {
        //     println!("{i}, {t:?}");
        // }
        //
        HuffmanTree {
            root: root.index,
            tree,
            symbols,
        }
    }

    pub(crate) fn get_code(&self, byte: u8) -> Vec<u8> {
        // println!("get_code: {byte:?}");
        let mut output = Vec::new();
        let mut node = self.tree[self.symbols[&byte]];

        loop {
            // println!("{node:?}");
            if let Some(p_idx) = node.parent {
                let p = self.tree[p_idx];
                if p.left.is_some_and(|l| l == node.index) {
                    // println!("left");
                    output.push(1);
                } else if p.right.is_some_and(|r| r == node.index) {
                    // println!("right");
                    output.push(0);
                }
                node = p;
            } else {
                break;
            }
        }

        output.into_iter().rev().collect()
    }

    fn decode(&self, code: &[u8]) -> u8 {
        let mut idx = self.root;
        for c in code {
            if *c == 1 {
                idx = self.tree[idx].left.unwrap();
            } else {
                idx = self.tree[idx].right.unwrap();
            }
        }

        self.tree[idx].symbol.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn it_works() {
        let text = "so much words wow many compression";
        let tree = HuffmanTree::new(text.as_bytes());
        tree.mem_dbg(DbgFlags::default()).unwrap();
        let chars: HashSet<&str> = HashSet::from_iter(text.split("").filter(|c| *c != ""));

        for c in chars {
            let actual_byte = c.as_bytes()[0];
            let actual_code = tree.get_code(actual_byte);
            let decoded_byte = tree.decode(&actual_code);
            // println!("c={c:?}, actual_byte={actual_byte:?}, decoded_byte={decoded_byte:?}, actual_code={actual_code:?}");
            assert_eq!(actual_byte, decoded_byte);
        }
    }
}
