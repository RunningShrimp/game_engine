//! # Fuzzy Matching Algorithm
//!
//! Provides fuzzy matching capabilities for code completion.

/// Fuzzy match result
#[derive(Debug, Clone, PartialEq)]
pub struct FuzzyMatch {
    /// Matched item
    pub item: String,

    /// Match score (0-1, higher is better)
    pub score: f64,

    /// Matched positions in the item
    pub positions: Vec<usize>,
}

/// Fuzzy matcher
pub struct FuzzyMatcher {
    /// Case sensitivity
    case_sensitive: bool,

    /// Minimum score threshold
    min_score: f64,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher
    pub fn new() -> Self {
        Self {
            case_sensitive: false,
            min_score: 0.3,
        }
    }

    /// Set case sensitivity
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, min_score: f64) -> Self {
        self.min_score = min_score;
        self
    }

    /// Find all fuzzy matches
    ///
    /// # Arguments
    ///
    /// * `pattern` - The search pattern
    /// * `items` - Items to search
    ///
    /// # Returns
    ///
    /// List of matches, sorted by score (descending)
    pub fn find_matches(&self, pattern: &str, items: &[String]) -> Vec<FuzzyMatch> {
        let mut matches = Vec::new();

        for item in items {
            if let Some(result) = self.fuzzy_match(pattern, item) {
                if result.score >= self.min_score {
                    matches.push(result);
                }
            }
        }

        // Sort by score (descending)
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        matches
    }

    /// Perform fuzzy matching
    ///
    /// # Arguments
    ///
    /// * `pattern` - The search pattern
    /// * `text` - The text to match against
    ///
    /// # Returns
    ///
    /// The match result, if the pattern matches
    pub fn fuzzy_match(&self, pattern: &str, text: &str) -> Option<FuzzyMatch> {
        let pattern_lower = if self.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let text_lower = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        if pattern.is_empty() {
            return Some(FuzzyMatch {
                item: text.to_string(),
                score: 1.0,
                positions: vec![],
            });
        }

        // Find all possible matches
        let mut best_match = None;
        let mut best_score = 0.0;

        // Try different matching strategies
        if let Some(result) = self.subsequence_match(&pattern_lower, &text_lower) {
            if result.score > best_score {
                best_match = Some(result);
                best_score = result.score;
            }
        }

        if let Some(result) = self.substring_match(&pattern_lower, &text_lower) {
            if result.score > best_score {
                best_match = Some(result);
                best_score = result.score;
            }
        }

        if let Some(result) = self.prefix_match(&pattern_lower, &text_lower) {
            if result.score > best_score {
                best_match = Some(result);
                best_score = result.score;
            }
        }

        best_match.map(|mut m| {
            m.item = text.to_string();
            m
        })
    }

    /// Subsequence matching
    fn subsequence_match(&self, pattern: &str, text: &str) -> Option<FuzzyMatch> {
        let mut positions = Vec::new();
        let mut pattern_idx = 0;
        let mut text_idx = 0;

        while pattern_idx < pattern.len() && text_idx < text.len() {
            if pattern.chars().nth(pattern_idx) == text.chars().nth(text_idx) {
                positions.push(text_idx);
                pattern_idx += 1;
            }
            text_idx += 1;
        }

        if pattern_idx == pattern.len() {
            // All pattern characters matched
            let score = Self::calculate_score(pattern, text, &positions);
            Some(FuzzyMatch {
                item: String::new(), // Will be set by caller
                score,
                positions,
            })
        } else {
            None
        }
    }

    /// Substring matching
    fn substring_match(&self, pattern: &str, text: &str) -> Option<FuzzyMatch> {
        if let Some(pos) = text.find(pattern) {
            let positions: Vec<usize> = (pos..pos + pattern.len()).collect();
            let score = Self::calculate_score(pattern, text, &positions);
            Some(FuzzyMatch {
                item: String::new(),
                score,
                positions,
            })
        } else {
            None
        }
    }

    /// Prefix matching
    fn prefix_match(&self, pattern: &str, text: &str) -> Option<FuzzyMatch> {
        if text.starts_with(pattern) {
            let positions: Vec<usize> = (0..pattern.len()).collect();
            let score = Self::calculate_score(pattern, text, &positions);
            Some(FuzzyMatch {
                item: String::new(),
                score,
                positions,
            })
        } else {
            None
        }
    }

    /// Calculate match score
    fn calculate_score(pattern: &str, text: &str, positions: &[usize]) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        // Base score from pattern coverage
        let coverage = positions.len() as f64 / pattern.len() as f64;

        // Bonus for consecutive matches
        let mut consecutive_bonus = 0.0;
        for i in 0..positions.len() - 1 {
            if positions[i] + 1 == positions[i + 1] {
                consecutive_bonus += 0.1;
            }
        }

        // Bonus for word boundaries
        let mut word_boundary_bonus = 0.0;
        for &pos in positions {
            if pos == 0 || text.chars().nth(pos - 1).map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                word_boundary_bonus += 0.15;
            }
        }

        // Penalty for gaps between matches
        let gap_penalty = if positions.len() > 1 {
            let total_gap = positions.last().unwrap() - positions.first().unwrap();
            let expected_gap = positions.len() - 1;
            if total_gap > expected_gap {
                (total_gap - expected_gap) as f64 * 0.05
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate final score
        let mut score = coverage + consecutive_bonus + word_boundary_bonus - gap_penalty;
        score = score.max(0.0).min(1.0);

        score
    }

    /// Get similarity score between two strings
    ///
    /// # Arguments
    ///
    /// * `s1` - First string
    /// * `s2` - Second string
    ///
    /// # Returns
    ///
    /// Similarity score (0-1, 1 being identical)
    pub fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }

        // Levenshtein distance
        let distance = self.levenshtein_distance(s1, s2);
        let max_len = s1.len().max(s2.len());

        if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len as f64)
        }
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = [
                    matrix[i - 1][j] + 1,        // deletion
                    matrix[i][j - 1] + 1,        // insertion
                    matrix[i - 1][j - 1] + cost, // substitution
                ]
                .iter()
                .min()
                .unwrap()
                .clone();
            }
        }

        matrix[len1][len2]
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matcher_creation() {
        let matcher = FuzzyMatcher::new();
        assert_eq!(matcher.min_score, 0.3);
        assert!(!matcher.case_sensitive);
    }

    #[test]
    fn test_prefix_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("Vec", "Vec");
        assert!(result.is_some());
        assert_eq!(result.unwrap().score, 1.0);
    }

    #[test]
    fn test_substring_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("ec", "Vector");
        assert!(result.is_some());
        assert!(result.unwrap().score > 0.5);
    }

    #[test]
    fn test_subsequence_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("Vtr", "Vector");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_matches() {
        let matcher = FuzzyMatcher::new();
        let items = vec![
            "Vector".to_string(),
            "VecDeque".to_string(),
            "HashMap".to_string(),
            "Value".to_string(),
        ];
        let matches = matcher.find_matches("Vec", &items);
        assert!(!matches.is_empty());
        // Vector and VecDeque should match
        assert!(matches.iter().any(|m| m.item == "Vector"));
        assert!(matches.iter().any(|m| m.item == "VecDeque"));
    }

    #[test]
    fn test_similarity() {
        let matcher = FuzzyMatcher::new();
        let sim = matcher.similarity("test", "test");
        assert_eq!(sim, 1.0);

        let sim = matcher.similarity("test", "toast");
        assert!(sim > 0.5 && sim < 1.0);
    }

    #[test]
    fn test_case_insensitive() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.fuzzy_match("vec", "VECTOR");
        assert!(result.is_some());
    }
}
