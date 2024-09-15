use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::Array2;
use regex::Regex;
use linfa_logistic::FittedLogisticRegression;

// Function to tokenize text using regex
fn tokenize(text: &str) -> Vec<String> {
    let re = Regex::new(r"\w+").unwrap();
    re.find_iter(text)
        .map(|mat| mat.as_str().to_lowercase())
        .collect()
}

// Convert tokens into feature vectors (bag-of-words representation)
fn vectorize(tokens: Vec<Vec<String>>, vocab: &Vec<String>) -> Array2<f64> {
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

// Function to predict if a new message is spam or not
fn predict_message(
    model: &FittedLogisticRegression<f64, usize>,
    vocab: &Vec<String>,
    message: &str,
) -> bool {
    let tokenized_message = tokenize(message);
    let data = vectorize(vec![tokenized_message], vocab);
    let prediction = model.predict(&data);
    prediction[0] == 1 // Returns true if spam (label 1)
}

fn main() {
    // Example text messages (spam and non-spam)
    let messages = vec![
        "Win $1000 cash now! Call this number!",
        "Hi, how are you doing?",
        "Exclusive offer just for you, claim your prize now!",
        "Are we meeting at 5 pm today?",
        "Congratulations, you've won a free vacation!",
        "Let's catch up later, maybe grab some coffee.",
    ];

    // Labels: 1 for spam, 0 for non-spam
    let labels = vec![1, 0, 1, 0, 1, 0];

    // Tokenize the messages
    let tokenized_messages: Vec<Vec<String>> = messages.iter().map(|msg| tokenize(msg)).collect();

    // Create a vocabulary from the tokenized messages
    let mut vocab: Vec<String> = tokenized_messages
        .iter()
        .flat_map(|tokens| tokens.iter().cloned())
        .collect();
    vocab.sort();
    vocab.dedup();

    // Convert the tokenized messages into a feature matrix
    let data = vectorize(tokenized_messages, &vocab);

    // Convert labels to ndarray
    let target = ndarray::Array::from_vec(labels);

    // Create a dataset and split it into training and test data (using all data for training)
    let dataset = linfa::dataset::DatasetBase::from((data, target));
    let (train_data, _) = dataset.split_with_ratio(1.0); // Use all data for training

    // Train a logistic regression model
    let model = LogisticRegression::default()
        .max_iterations(100)
        .fit(&train_data)
        .expect("Failed to fit model");

    // Test the model with a new example message
    let new_message = "Claim your exclusive reward now!";
    let is_spam = predict_message(&model, &vocab, new_message);
    println!(
        "Prediction for message: '{}': {}",
        new_message,
        if is_spam { "Spam" } else { "Not Spam" }
    );

    // Test another message
    let new_message_2 = "Hey, let's meet for lunch tomorrow.";
    let is_spam_2 = predict_message(&model, &vocab, new_message_2);
    println!(
        "Prediction for message: '{}': {}",
        new_message_2,
        if is_spam_2 { "Spam" } else { "Not Spam" }
    );
}
