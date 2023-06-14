use bitvec::prelude as bv;
use huff_coding::prelude::*;

use std::ops::Index;

#[derive(Debug)]
pub struct Sfdc<L: HuffLetter> {
    text: Vec<L>,
    tree: HuffTree<L>,
    layers: Vec<SfdcLayer>,
    dynamic_layer: SfdcLayer,
}

type SfdcLayer = bv::BitVec;

impl<L: HuffLetter> Sfdc<L> {
    pub fn new(text: &[L], layers: usize) -> Self {
        let weights = build_weights_map(text);
        let tree = HuffTree::from_weights(weights);
        let codes = tree.read_codes();

        let max_code_length = codes.values().map(|v| v.len()).max().unwrap();
        let layers = if layers <= 1 {
            2
        } else if layers > max_code_length {
            max_code_length
        } else {
            layers
        };

        let layers = vec![SfdcLayer::repeat(false, text.len()); layers];

        Self {
            text: text.into(),
            tree,
            layers,
            // TODO: Better to allocate a bigger chunk now? We could probably guess at that based
            // on the number of layers requested, and the frequency of characters with coes longer
            // than the number of layers...
            dynamic_layer: SfdcLayer::repeat(false, text.len()),
        }
    }

    pub fn encode(&mut self) {
        let codes = self.tree.read_codes();
        let mut pending = SfdcLayer::new();

        for (i, c) in self.text.iter().enumerate() {
            let code = &codes[c];
            let fixed_idx = std::cmp::min(code.len(), self.layers.len());

            for j in 0..fixed_idx {
                self.layers[j].set(i, code[j]);
            }

            for j in (fixed_idx..code.len()).rev() {
                pending.push(code[j]);
            }

            if let Some(b) = pending.pop() {
                self.dynamic_layer.set(i, b);
            }
        }

        while let Some(b) = pending.pop() {
            self.dynamic_layer.push(b);
        }
    }

    pub fn decode_one(&self, index: usize) -> &L {
        self.decode_range(index, index)[0]
    }

    pub fn decode_range(&self, start: usize, end: usize) -> Vec<&L> {
        let start = if start >= self.text.len() {
            self.text.len() - 1
        } else {
            start
        };

        let end = if end >= self.text.len() {
            self.text.len() - 1
        } else {
            end
        };

        let expected = std::cmp::max(1, (end - start) + 1);
        let mut pending = Vec::new();
        let mut results: Vec<Option<&L>> = vec![None; expected];
        let mut k = start;
        let mut found = 0;

        while found < expected {
            if k < self.text.len() {
                let mut x = self.tree.root();
                let mut h = 0;
                while h < self.layers.len() && x.has_children() {
                    if self.layers[h][k] {
                        x = x.right_child().unwrap();
                    } else {
                        x = x.left_child().unwrap();
                    }

                    h += 1;
                }

                if !x.has_children() {
                    if k <= end {
                        found += 1;
                        results[k - start] = x.leaf().letter();
                    }
                } else {
                    pending.push((x, k));
                }
            }

            if let Some((mut x, p)) = pending.pop() {
                if self.dynamic_layer[k] {
                    x = x.right_child().unwrap();
                } else {
                    x = x.left_child().unwrap();
                }

                if !x.has_children() {
                    if p <= end {
                        found += 1;
                        results[p - start] = x.leaf().letter();
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

impl<L: HuffLetter> Index<usize> for Sfdc<L> {
    type Output = L;
    fn index(&self, index: usize) -> &Self::Output {
        self.decode_one(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes_range() {
        let texts = vec!["Compression", "The quick brown fox jumps over the lazy dog"];

        let layers = 2..=5;

        for n in layers {
            for t in &texts {
                let text: Vec<&str> = t.split("").collect();
                let text = &text[1..text.len() - 1];

                let mut sfdc = Sfdc::new(text, n);
                sfdc.encode();

                let decoded = sfdc.decode_range(0, text.len());
                for (i, c) in text.iter().enumerate() {
                    assert_eq!(c, decoded[i]);
                }
            }
        }
    }

    #[test]
    fn it_decodes_one() {
        let text: Vec<&str> = "Compression".split("").collect();
        let text = &text[1..text.len() - 1];

        let layers = 2..=5;

        for n in layers {
            let mut sfdc = Sfdc::new(text, n);
            sfdc.encode();

            for (i, c) in text.iter().enumerate() {
                assert_eq!(c, sfdc.decode_one(i));
            }
        }
    }

    #[test]
    fn it_works_integers() {
        let n_layers = 3;

        let text: Vec<u64> = vec![u64::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u64::MAX];
        let mut sfdc = Sfdc::new(&text, n_layers);
        sfdc.encode();
        let decoded = sfdc.decode_range(0, 0);
        assert_eq!(&u64::MIN, decoded[0]);
        let decoded = sfdc.decode_one(text.len());
        assert_eq!(&u64::MAX, decoded);

        let text: Vec<i64> = vec![i64::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, i64::MAX];
        let mut sfdc = Sfdc::new(&text, n_layers);
        sfdc.encode();
        let decoded = sfdc.decode_range(0, 0);
        assert_eq!(&i64::MIN, decoded[0]);
        let decoded = sfdc.decode_one(text.len());
        assert_eq!(&i64::MAX, decoded);

        let text: Vec<u32> = vec![u32::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u32::MAX];
        let mut sfdc = Sfdc::new(&text, n_layers);
        sfdc.encode();
        let decoded = sfdc.decode_range(0, 0);
        assert_eq!(&u32::MIN, decoded[0]);
        let decoded = sfdc.decode_one(text.len());
        assert_eq!(&u32::MAX, decoded);

        let text: Vec<i32> = vec![i32::MIN, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, i32::MAX];
        let mut sfdc = Sfdc::new(&text, n_layers);
        sfdc.encode();
        let decoded = sfdc.decode_range(0, 0);
        assert_eq!(&i32::MIN, decoded[0]);
        let decoded = sfdc.decode_one(text.len());
        assert_eq!(&i32::MAX, decoded);
    }

    #[test]
    fn it_implements_index() {
        let text = vec![0, 1, 2, 3, 4];
        let mut sfdc = Sfdc::new(&text, 3);
        sfdc.encode();

        for (i, n) in text.iter().enumerate() {
            assert_eq!(*n, sfdc[i]);
        }
    }
}
