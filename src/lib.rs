mod huffman;

use bitvec::prelude as bv;
use mem_dbg::*;

use huffman::*;

use std::ops::Index;

#[derive(Clone, Debug, MemSize)]
pub struct Sfdc {
    len: usize,
    tree: HuffmanTree,
    layers: Vec<SfdcLayer>,
    dynamic_layer: SfdcLayer,
}

#[derive(Debug, Clone)]
pub struct SfdcLayer(bv::BitVec);

impl SfdcLayer {
    pub fn new() -> Self {
        Self(bv::BitVec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.0.set(index, value)
    }

    pub fn push(&mut self, value: bool) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Option<bool> {
        self.0.pop()
    }
}

impl Index<usize> for SfdcLayer {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl MemSize for SfdcLayer {
    fn mem_size(&self, _flags: SizeFlags) -> usize {
        self.0.as_raw_slice().len() * std::mem::size_of::<usize>()
    }
}

impl CopyType for SfdcLayer {
    type Copy = False;
}

impl Sfdc {
    pub fn new(text: &[u8], layers: usize) -> Self {
        let tree = HuffmanTree::new(text);
        // let codes = tree.read_codes();
        // let max_code_length = codes.values().map(|v| v.len()).max().unwrap();
        // let layers = if layers <= 1 {
        //     2
        // } else if layers > max_code_length {
        //     max_code_length
        // } else {
        //     layers
        // };

        let layers = vec![SfdcLayer(bv::BitVec::repeat(false, text.len())); layers.max(2)];

        let mut sfdc = Self {
            len: layers[0].len(),
            tree,
            layers,
            // TODO: Better to allocate a bigger chunk now? We could probably guess at that based
            // on the number of layers requested, and the frequency of characters with coes longer
            // than the number of layers...
            dynamic_layer: SfdcLayer(bv::BitVec::repeat(false, text.len())),
        };

        sfdc.encode(text);

        sfdc
    }

    fn encode(&mut self, text: &[u8]) {
        // let codes = self.tree.read_codes();
        let mut pending = SfdcLayer::new();

        for (i, c) in text.iter().enumerate() {
            let code = self.tree.get_code(*c);
            let fixed_idx = std::cmp::min(code.len(), self.layers.len());

            for j in 0..fixed_idx {
                self.layers[j].set(i, code[j] == 1);
            }

            for j in (fixed_idx..code.len()).rev() {
                pending.push(code[j] == 1);
            }

            if let Some(b) = pending.pop() {
                self.dynamic_layer.set(i, b);
            }
        }

        while let Some(b) = pending.pop() {
            self.dynamic_layer.push(b);
        }
    }

    pub fn decode_one(&self, index: usize) -> u8 {
        self.decode_range(index, index)[0]
    }

    pub fn decode_range(&self, start: usize, end: usize) -> Vec<u8> {
        let start = if start >= self.len {
            self.len - 1
        } else {
            start
        };

        let end = if end >= self.len { self.len - 1 } else { end };

        let expected = std::cmp::max(1, (end - start) + 1);
        let mut pending = Vec::new();
        let mut results: Vec<Option<u8>> = vec![None; expected];
        let mut k = start;
        let mut found = 0;

        while found < expected {
            if k < self.len {
                let mut x = self.tree.tree[self.tree.root];
                let mut h = 0;
                while h < self.layers.len() && (x.left.is_some() || x.right.is_some()) {
                    if self.layers[h][k] {
                        x = self.tree.tree[x.left.unwrap()];
                    } else {
                        x = self.tree.tree[x.right.unwrap()];
                    }

                    h += 1;
                }

                if x.left.is_none() && x.right.is_none() {
                    if k <= end {
                        found += 1;
                        results[k - start] = Some(self.tree.tree[x.index].index as u8);
                    }
                } else {
                    pending.push((x, k));
                }
            }

            if let Some((mut x, p)) = pending.pop() {
                if self.dynamic_layer[k] {
                    x = self.tree.tree[x.left.unwrap()];
                } else {
                    x = self.tree.tree[x.right.unwrap()];
                }

                if x.left.is_none() && x.right.is_none() {
                    if p <= end {
                        found += 1;
                        results[p - start] = Some(x.index as u8);
                    }
                } else {
                    pending.push((x, p));
                }
            }

            k += 1;
        }

        results.iter().map(|r| r.unwrap()).collect()
    }
}

// impl Index<usize> for Sfdc {
//     type Output = u8;
//     fn index(&self, index: usize) -> &Self::Output {
//         self.decode_one(index)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes_range() {
        let texts = vec!["Compression", "The quick brown fox jumps over the lazy dog"];

        let layers = 2..=5;

        for n in layers {
            for t in &texts {
                let text = t.as_bytes();
                let sfdc = Sfdc::new(text, n);

                let decoded = sfdc.decode_range(0, text.len());
                for (i, c) in text.iter().enumerate() {
                    assert_eq!(*c, decoded[i]);
                }
            }
        }
    }

    #[test]
    fn it_decodes_one() {
        let text = "Compression".as_bytes();
        let layers = 2..=5;

        for n in layers {
            let sfdc = Sfdc::new(text, n);

            for (i, c) in text.iter().enumerate() {
                assert_eq!(*c, sfdc.decode_one(i));
            }
        }
    }

    // #[test]
    // fn it_works_integers() {
    //     let n_layers = 3;
    //
    //     let text: Vec<u64> = vec![u64::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u64::MAX];
    //     let sfdc = Sfdc::new(&text, n_layers);
    //     let decoded = sfdc.decode_range(0, 0);
    //     assert_eq!(&u64::MIN, decoded[0]);
    //     let decoded = sfdc.decode_one(text.len());
    //     assert_eq!(&u64::MAX, decoded);
    //
    //     let text: Vec<i64> = vec![i64::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, i64::MAX];
    //     let sfdc = Sfdc::new(&text, n_layers);
    //     let decoded = sfdc.decode_range(0, 0);
    //     assert_eq!(&i64::MIN, decoded[0]);
    //     let decoded = sfdc.decode_one(text.len());
    //     assert_eq!(&i64::MAX, decoded);
    //
    //     let text: Vec<u32> = vec![u32::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u32::MAX];
    //     let sfdc = Sfdc::new(&text, n_layers);
    //     let decoded = sfdc.decode_range(0, 0);
    //     assert_eq!(&u32::MIN, decoded[0]);
    //     let decoded = sfdc.decode_one(text.len());
    //     assert_eq!(&u32::MAX, decoded);
    //
    //     let text: Vec<i32> = vec![i32::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, i32::MAX];
    //     let sfdc = Sfdc::new(&text, n_layers);
    //     let decoded = sfdc.decode_range(0, 0);
    //     assert_eq!(&i32::MIN, decoded[0]);
    //     let decoded = sfdc.decode_one(text.len());
    //     assert_eq!(&i32::MAX, decoded);
    // }

    // #[test]
    // fn it_implements_index() {
    //     let text = vec![0, 1, 2, 3, 4];
    //     let sfdc = Sfdc::new(&text, 3);
    //
    //     for (i, n) in text.iter().enumerate() {
    //         assert_eq!(*n, sfdc[i]);
    //     }
    // }
}
