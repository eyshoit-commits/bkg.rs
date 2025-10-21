use crate::core::schema::QueryResult;

pub fn normalize_scores(results: &mut [QueryResult]) {
    if results.is_empty() {
        return;
    }
    let max_score = results
        .iter()
        .map(|result| result.score)
        .fold(f32::MIN, f32::max);
    if max_score <= 0.0 {
        return;
    }
    for result in results.iter_mut() {
        result.score /= max_score;
    }
}
