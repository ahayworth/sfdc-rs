use bitvec::prelude as bv;
use huff_coding::prelude::*;

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
        if layers <= 1 || layers > max_code_length {
            panic!(
                "Layers must be > 1 and <= max code length! (got layers: {}, max code length: {})",
                layers, max_code_length
            );
        }

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

    pub fn decode(&self, start: usize, end: usize) -> Vec<&L> {
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
                        println!("{:?}", results);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_short() {
        let text = ["C", "o", "m", "p", "r", "e", "s", "s", "i", "o", "n"];
        let n_layers = 2;
        let mut sfdc = Sfdc::new(&text, n_layers);

        assert_eq!(n_layers, sfdc.layers.len());

        sfdc.encode();
        for (k, v) in sfdc.tree.read_codes() {
            println!("{k} {:?}", v.to_string());
        }

        for i in 0..n_layers {
            println!("(layer {i}): {:?}", sfdc.layers[i]);
        }
        println!("(layer d): {:?}", sfdc.dynamic_layer);
        let decoded = sfdc.decode(0, text.len() - 1);

        for (i, c) in text.iter().enumerate() {
            assert_eq!(c, decoded[i]);
        }

        let decoded = sfdc.decode(0, 0);
        assert_eq!(&"C", decoded[0]);

        let decoded = sfdc.decode(10, 11);
        assert_eq!(&"n", decoded[0]);
    }

    #[test]
    fn it_works_long() {
        let text: Vec<&str> = "The quick brown fox jumps over the lazy dog"
            .split("")
            .collect();
        let text = &text[1..text.len() - 1];
        let n_layers = 3;
        let mut sfdc = Sfdc::new(&text, n_layers);

        assert_eq!(n_layers, sfdc.layers.len());

        sfdc.encode();

        for (k, v) in sfdc.tree.read_codes() {
            println!("{k} {:?}", v.to_string());
        }

        for i in 0..n_layers {
            println!("(layer {i}): {:?}", sfdc.layers[i]);
        }
        println!("(layer d): {:?}", sfdc.dynamic_layer);

        let decoded = sfdc.decode(0, text.len());

        for (i, c) in text.iter().enumerate() {
            assert_eq!(c, decoded[i]);
        }

        let decoded = sfdc.decode(0, 0);
        assert_eq!(&"T", decoded[0]);

        let decoded = sfdc.decode(4, 5);
        assert_eq!(&"q", decoded[0]);
    }

    #[test]
    fn it_works_integers() {
        let text: Vec<u64> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 100_000_000];
        let n_layers = 3;
        let mut sfdc = Sfdc::new(&text, n_layers);

        assert_eq!(n_layers, sfdc.layers.len());

        sfdc.encode();

        for (k, v) in sfdc.tree.read_codes() {
            println!("{k} {:?}", v.to_string());
        }

        for i in 0..n_layers {
            println!("(layer {i}): {:?}", sfdc.layers[i]);
        }
        println!("(layer d): {:?}", sfdc.dynamic_layer);

        let decoded = sfdc.decode(0, 0);
        assert_eq!(&0, decoded[0]);

        let decoded = sfdc.decode(text.len(), text.len());
        assert_eq!(&100_000_000, decoded[0]);
    }
}
