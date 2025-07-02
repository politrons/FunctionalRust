use csv::Reader;
use itertools::Itertools;
use linfa::prelude::*;
use ndarray::{Array1, Array2, ArrayView1, Axis};
use serde::Deserialize;
use std::{collections::HashMap, error::Error};
use linfa_logistic::{LogisticRegression};

const CSV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/hello_rust.csv");
const EXAMPLES: &[&str] = &[
    "fn main() { println!(\"Hello, world!\"); }",
    "fn mian() { println!(\"Hello, world!\"); }",
    "fn main() { println!(\"Hola, mundo!\"); }",

];

const NGRAM_MIN: usize = 1;
const NGRAM_MAX: usize = 6;


fn token_ngrams(text: &str, n_min: usize, n_max: usize) -> Vec<String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();       
    (n_min..=n_max)
        .flat_map(|n| {
            tokens
                .windows(n)
                .map(|w| w.join(" "))                             
                .collect::<Vec<_>>()
        })
        .collect()
}


struct Vocab {
    index: HashMap<String, usize>,
}

impl Vocab {
    fn new(corpus: &[String]) -> Self {
        let mut index = HashMap::new();
        for doc in corpus {
            for ng in token_ngrams(doc, NGRAM_MIN, NGRAM_MAX) {
                let next = index.len();         
                index.entry(ng).or_insert(next);
            }

        }
        Vocab { index }
    }

    fn vectorize(&self, text: &str) -> Array1<f64> {
        let mut v = vec![0.0; self.index.len()];
        for ng in token_ngrams(text, NGRAM_MIN, NGRAM_MAX) {
            if let Some(&i) = self.index.get(&ng) {
                v[i] = 1.0;              
            }
        }
        Array1::from(v)
    }
}

#[derive(Deserialize)]
struct CsvRow {
    code: String,
    label: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load dataset
    let mut rdr = Reader::from_path(CSV_PATH)?;
    let mut snippets = Vec::<String>::new();
    let mut labels = Vec::<i32>::new();
    for row in rdr.deserialize::<CsvRow>() {
        let rec = row?;
        snippets.push(rec.code);
        labels.push(rec.label as i32);
    }
    println!("Dataset loaded: {} samples", snippets.len());

    // 2. Vectorize
    let vocab = Vocab::new(&snippets);
    let n_samples = snippets.len();
    let n_features = vocab.index.len();
    let mut x = Array2::zeros((n_samples, n_features));
    for (i, snippet) in snippets.iter().enumerate() {
        x.row_mut(i).assign(&vocab.vectorize(snippet));
    }
    let y = Array1::from(labels);

    // 3. Train model
    let (train, test) = Dataset::new(x, y).split_with_ratio(0.8);
    let model = LogisticRegression::default().fit(&train).unwrap();


    // Calculate simple accuracy manually
    let preds = model.predict(&test);
    let correct = preds
        .iter()
        .zip(test.targets().iter())
        .filter(|(p, t)| p == t)
        .count();
    let acc = correct as f64 / preds.len() as f64;
    println!("Test accuracy: {:.3}\n", acc);

    // 4. Predict examples
    for &snippet in EXAMPLES {
        let vec = vocab.vectorize(snippet);
        let arr2 = vec.insert_axis(Axis(0));
        let proba = model.predict_probabilities(&arr2)[0];
        println!("{:<70} → {} (p={:.3})",
                 snippet,
                 if proba >= 0.5 { "VALID" } else { "INVALID" },
                 proba);
    }

    Ok(())
}
