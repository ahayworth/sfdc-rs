use std::cmp::Reverse;
use std::collections::BinaryHeap;

use mem_dbg::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, MemSize, MemDbg)]
pub(crate) struct HuffmanNode {
    pub(crate) count: usize,
    pub(crate) index: usize,
    pub(crate) parent: Option<usize>,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, MemSize, MemDbg)]
pub(crate) struct HuffmanTree {
    pub(crate) root: usize,
    pub(crate) tree: Vec<HuffmanNode>,
}

impl HuffmanTree {
    pub(crate) fn new(input: &[u8]) -> HuffmanTree {
        let mut tree: Vec<HuffmanNode> = (0..u8::MAX as usize)
            .map(|i| HuffmanNode {
                index: i,
                ..Default::default()
            })
            .collect();

        for t in input {
            let i: usize = usize::from(*t);
            tree[i].count += 1;
            tree[i].index = i;
        }

        let mut heap = BinaryHeap::from_iter(
            tree.iter()
                .filter(|n| n.count > 0)
                .map(|n| Reverse(n.clone())),
        );

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

        HuffmanTree {
            root: root.index,
            tree,
        }
    }

    pub(crate) fn get_code(&self, byte: u8) -> Vec<u8> {
        // println!("get_code: {byte:?}");
        let mut output = Vec::new();
        let mut node = self.tree[usize::from(byte)];

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

        return idx as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // TODO: test prefix table matches expectations

    #[test]
    fn it_works() {
        let text = "so much words wow many compression";
        let tree = HuffmanTree::new(text.as_bytes());
        let chars: HashSet<&str> = HashSet::from_iter(text.split("").filter(|c| *c != ""));

        for c in chars {
            let actual_byte = c.as_bytes()[0];
            let actual_code = tree.get_code(actual_byte);
            let decoded_byte = tree.decode(&actual_code);
            println!("c={c:?}, actual_byte={actual_byte:?}, decoded_byte={decoded_byte:?}, actual_code={actual_code:?}");
            assert_eq!(actual_byte, decoded_byte);
        }

        println!("{:?}", tree.mem_size(SizeFlags::default()));
        println!("{:?}", text.mem_size(SizeFlags::default()));
        tree.mem_dbg(DbgFlags::default()).unwrap();
    }
}
