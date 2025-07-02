use csv::Reader;
use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::{Array1, Array2, Axis};
use serde::Deserialize;
use std::{collections::HashMap, error::Error};

// ---------------------------------------------------------------------
// Constants and configuration
// ---------------------------------------------------------------------
const CSV_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/hello_rust.csv");
const EXAMPLES: &[&str] = &[
    "fn main() { println!(\"Hello, world!\"); }", // correct
    "fn mian() { println!(\"Hello, world!\"); }", // misspelled `main`
    "fn main() { println!(\"Hola, mundo!\"); }",   // wrong string literal
];
const NGRAM_MIN: usize = 1;
const NGRAM_MAX: usize = 6; // token n‑grams from 1‑ to 6‑grams

// ---------------------------------------------------------------------
// Token n‑gram extractor → Vec<String>
// ---------------------------------------------------------------------
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

// ---------------------------------------------------------------------
// Vocabulary = token‑ngram → column index   +   vectorizer (one‑hot)
// ---------------------------------------------------------------------
struct Vocab {
    index: HashMap<String, usize>,
}

impl Vocab {
    /// Build a global vocabulary from the full corpus.
    fn new(corpus: &[String]) -> Self {
        let mut index = HashMap::new();
        for doc in corpus {
            for ng in token_ngrams(doc, NGRAM_MIN, NGRAM_MAX) {
                let next_idx = index.len();
                index.entry(ng).or_insert(next_idx);
            }
        }
        Vocab { index }
    }

    /// Convert a snippet into a dense presence/absence vector (f64).
    fn vectorize(&self, text: &str) -> Array1<f64> {
        let mut v = vec![0.0; self.index.len()];
        for ng in token_ngrams(text, NGRAM_MIN, NGRAM_MAX) {
            if let Some(&i) = self.index.get(&ng) {
                v[i] = 1.0; // binary one‑hot
            }
        }
        Array1::from(v)
    }
}

// CSV → struct (serde)
#[derive(Deserialize)]
struct CsvRow {
    code: String,
    label: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    // ================================================================
    // LOAD DATASET
    // ---------------------------------------------------------------
    // • Open the CSV and deserialize each line into `CsvRow`.
    // • Push snippet texts into `snippets` and labels (0/1) into `labels`.
    //   At the end we have two aligned vectors ready for feature extraction.
    // ================================================================
    let mut rdr = Reader::from_path(CSV_PATH)?;
    let mut snippets = Vec::<String>::new();
    let mut labels = Vec::<i32>::new();

    for row in rdr.deserialize::<CsvRow>() {
        let rec = row?;
        snippets.push(rec.code);
        labels.push(rec.label as i32);
    }
    println!("Dataset loaded: {} samples", snippets.len());

    // ================================================================
    // VECTORIZATION
    // ---------------------------------------------------------------
    // • Build the global vocabulary from ALL snippets.
    // • Turn each snippet into a one‑hot vector; stack into matrix `x`.
    // • Convert labels into 1‑D array `y`.
    // ================================================================
    let vocab = Vocab::new(&snippets);
    let n_samples = snippets.len();
    let n_features = vocab.index.len();

    let mut x = Array2::<f64>::zeros((n_samples, n_features));
    for (i, snippet) in snippets.iter().enumerate() {
        x.row_mut(i).assign(&vocab.vectorize(snippet));
    }
    let y = Array1::from(labels);

    // ================================================================
    // TRAIN MODEL
    // ---------------------------------------------------------------
    // • Wrap (x,y) into Linfa `Dataset` and split 80/20.
    // • • train → 80 % of the rows (used to fit/learn the model).
    // • • test → 20 % of the rows (held-out set to evaluate accuracy).
    // • Fit a binary Logistic Regression (no regularization in linfa‑logistic).
    // • Compute plain accuracy by comparing predictions with ground truth.
    // ================================================================
    let (train, test) = Dataset::new(x, y).split_with_ratio(0.8);

    let model = LogisticRegression::default()
        .fit(&train)?;

    // manual accuracy
    let preds = model.predict(&test);
    let correct = preds
        .iter()
        .zip(test.targets().iter())
        .filter(|(p, t)| p == t)
        .count();
    let acc = correct as f64 / preds.len() as f64;
    println!("Test accuracy: {:.3}\n", acc);

    // ================================================================
    // PREDICT EXAMPLES
    // ---------------------------------------------------------------
    // • Vectorize three hard‑coded snippets.
    // • Use `predict_probabilities` (class‑1 probs) to show verdict.
    // ================================================================
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
