use crate::core::schema::QueryResult;
use std::collections::HashMap;
use tracing::instrument;

#[instrument(skip_all, fields(candidates = results.len(), rrf_k = rrf_k))]
pub fn reciprocal_rank_fusion(mut results: Vec<QueryResult>, rrf_k: usize) -> Vec<QueryResult> {
    let mut scores: HashMap<String, f32> = HashMap::new();
    for result in &results {
        let rank = result.rank as f32;
        let contribution = 1.0 / (rrf_k as f32 + rank);
        *scores.entry(result.id.clone()).or_insert(0.0) += contribution;
    }
    results.sort_by(|a, b| {
        let score_a = scores.get(&a.id).copied().unwrap_or(0.0);
        let score_b = scores.get(&b.id).copied().unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (idx, result) in results.iter_mut().enumerate() {
        result.rank = idx + 1;
        result.score = scores.get(&result.id).copied().unwrap_or(0.0);
    }
    results
}
