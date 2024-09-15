use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::{Array2, Ix1};
use regex::Regex;
use linfa_logistic::FittedLogisticRegression;

/// # Sentiment Analysis Example using Linfa
///
/// This Rust program demonstrates how to perform sentiment analysis using the
/// `linfa` crate for machine learning. We use logistic regression to classify
/// short text reviews as positive or negative sentiments.
///
/// ## Overview
/// - **Tokenization**: Convert the input text into individual words (tokens).
/// - **Bag-of-Words Vectorization**: Represent tokenized words as numerical
///   features for the model.
/// - **Logistic Regression**: Train a logistic regression model using the
///   `linfa_logistic` crate to classify the sentiment of reviews.
///
/// ## Crates Used
/// - `linfa`: General machine learning tasks.
/// - `linfa_logistic`: Logistic regression implementation.
/// - `ndarray`: For creating and managing feature matrices.
/// - `regex`: To tokenize the input text.
///
/// ## How to Use
/// 1. Modify the `reviews` and `labels` arrays to test different text data.
/// 2. Run the program to train the model and predict sentiment on new reviews.

fn main() {
    // Example text reviews (positive and negative)
    let reviews_to_train_model = vec![
        "I love this product, it's amazing!",
        "This is the worst purchase I have ever made. A totally waste",
        "Excellent quality and fast shipping.",
        "The product broke after just one use.",
        "I'm very happy with my order.",
        "Terrible service and rude staff.",
        "The shipping was fast but the product terrible.",
    ];

    // Labels: 1 for positive, 0 for negative
    let labels = vec![1, 0, 1, 0, 1, 0, 0];

    // Tokenize the reviews
    let tokenized_reviews: Vec<Vec<String>> = reviews_to_train_model.iter()
        .map(|review| tokenize(review))
        .collect();

    let vocab: Vec<String> = create_vocabulary_from_reviews(&tokenized_reviews);

    let data = convert_to_features(tokenized_reviews, &vocab);

    // Create a dataset and split it into training and test data (using all data for training)
    let dataset = linfa::dataset::DatasetBase::from((data, ndarray::Array::from_vec(labels)));
    let (train_data, _) = dataset.split_with_ratio(1.0); // Use all data for training

    let model = train_model(&train_data);

    let reviews = vec!["This is an amazing product!", "This was a waste of money.", "Dont  waste time and buy it, is amazing."];
    reviews.iter().for_each(|new_review| {
        let is_positive = predict_review(&model, &vocab, new_review);
        println!("Prediction for review: '{}': {}", new_review, if is_positive { "Positive" } else { "Negative" });
    });
}

// Create a vocabulary from the tokenized reviews
fn create_vocabulary_from_reviews(tokenized_reviews: &Vec<Vec<String>>) -> Vec<String> {
    let mut vocab: Vec<String> = tokenized_reviews
        .iter()
        .flat_map(|tokens| tokens.iter().cloned())
        .collect();
    vocab.sort();
    vocab.dedup();
    vocab
}

// Train a logistic regression model
fn train_model(train_data: &Dataset<f64, usize, Ix1>) -> FittedLogisticRegression<f64, usize> {
    LogisticRegression::default()
        .max_iterations(100)
        .fit(&train_data)
        .expect("Failed training model")
}

// Function to tokenize text using regex
fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"\w+").unwrap();
    re.find_iter(text)
        .map(|mat| mat.as_str().to_lowercase())
        .collect()
}

// Convert tokens into feature vectors (bag-of-words representation)
fn convert_to_features(tokens: Vec<Vec<String>>, vocab: &Vec<String>) -> Array2<f64> {
    let num_samples = tokens.len();
    let vocab_size = vocab.len();

    let mut array = Array2::zeros((num_samples, vocab_size));
    for (i, sample) in tokens.iter().enumerate() {
        for word in sample {
            if let Some(index) = vocab.iter().position(|vocab_word| vocab_word == word) {
                array[[i, index]] += 1.0;
            }
        }
    }
    array
}

// Function to predict sentiment of a new message (positive or negative)
fn predict_review(
    model: &FittedLogisticRegression<f64, usize>,
    vocab: &Vec<String>,
    message: &str,
) -> bool {
    let tokenized_message = tokenize(message);
    let data = convert_to_features(vec![tokenized_message], vocab);
    let prediction = model.predict(&data);
    prediction[0] == 1 // Returns true if positive sentiment (label 1)
}
