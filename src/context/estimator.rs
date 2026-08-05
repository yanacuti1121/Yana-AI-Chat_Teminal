// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenEstimator {
    chars_per_token: usize,
}

impl Default for TokenEstimator {
    fn default() -> Self {
        Self { chars_per_token: 4 }
    }
}

impl TokenEstimator {
    pub fn new(chars_per_token: usize) -> Self {
        Self {
            chars_per_token: chars_per_token.max(1),
        }
    }

    pub fn estimate(&self, text: &str) -> usize {
        let characters = text.chars().count();
        characters.div_ceil(self.chars_per_token).max(1)
    }

    pub fn estimate_many<'a>(&self, texts: impl IntoIterator<Item = &'a str>) -> usize {
        texts.into_iter().map(|text| self.estimate(text)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_is_deterministic_and_non_zero() {
        let estimator = TokenEstimator::default();
        assert_eq!(estimator.estimate("abcdefgh"), 2);
        assert_eq!(estimator.estimate(""), 1);
    }
}
