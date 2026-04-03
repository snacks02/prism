use std::collections::HashSet;

fn to_trigrams(value: &str) -> HashSet<(char, char, char)> {
    "  ".chars()
        .chain(value.chars())
        .chain("  ".chars())
        .collect::<Vec<char>>()
        .windows(3)
        .map(|window| (window[0], window[1], window[2]))
        .collect()
}

pub fn top_indexes(query: &str, values: &[String], threshold: f32) -> Vec<usize> {
    if query.is_empty() {
        return (0..values.len()).collect();
    }
    let query = query.to_lowercase();
    let query_trigrams = to_trigrams(&query);
    let similarity_scores: Vec<f32> = values
        .iter()
        .map(|value| {
            let value_trigrams = to_trigrams(&value.to_lowercase());
            let intersection = value_trigrams.intersection(&query_trigrams).count();
            2.0 * intersection as f32 / (value_trigrams.len() + query_trigrams.len()) as f32
        })
        .collect();
    let mut indexes: Vec<usize> = (0..values.len())
        .filter(|&index| similarity_scores[index] > threshold)
        .collect();
    indexes.sort_unstable_by(|&left, &right| {
        similarity_scores[right].total_cmp(&similarity_scores[left])
    });
    indexes
}
