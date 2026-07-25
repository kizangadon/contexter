//! Distance and similarity kernels for vector comparison.
//!
//! Provides the three core distance metrics used by the vector index:
//! cosine similarity, Euclidean (L2) distance, and dot product.

/// Cosine similarity: `dot(a, b) / (|a| * |b|)`.
///
/// Returns a value in [−1, 1] where 1 means identical direction, 0 means
/// orthogonal, and −1 means opposite direction.  If either vector has zero
/// magnitude, returns 0.0.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Euclidean (L2) distance: `sqrt(sum((a_i - b_i)^2))`.
///
/// Returns a non-negative value where 0 means identical vectors.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Dot product: `sum(a_i * b_i)`.
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = [1.0, 0.0, 0.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = [1.0, 0.0, 0.0];
        let b = [-1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_magnitude() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_euclidean_distance_zero() {
        let v = [1.0, 2.0, 3.0];
        let dist = euclidean_distance(&v, &v);
        assert!((dist - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_euclidean_distance_positive() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let dp = dot_product(&a, &b);
        assert!((dp - 32.0).abs() < f32::EPSILON); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_dot_product_zero() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 2.0, 3.0];
        let dp = dot_product(&a, &b);
        assert!((dp - 0.0).abs() < f32::EPSILON);
    }
}
