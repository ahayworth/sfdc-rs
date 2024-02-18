use mem_dbg::*;
use sfdc::*;

fn main() {
    let texts = vec![
        "Compression",
        "Absolute power corrupts absolutely",
        "The quick brown fox jumped over the lazy dog",
        "Those who cannot remember the past are condemned to repeat it",
    ];

    let texts = texts
        .iter()
        .map(|t| {
            let t = t.split("").collect::<Vec<_>>();
            t[1..t.len() - 1].to_vec()
        })
        .collect::<Vec<_>>();

    // for t in texts {
    //     let sfdc = Sfdc::new(&t, 3);
    //     println!("{:?}", sfdc.mem_size(SizeFlags::default()));
    // }
}
