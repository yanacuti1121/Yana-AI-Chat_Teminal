// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionPlan {
    batch_sizes: Vec<usize>,
}

impl Default for ExpansionPlan {
    fn default() -> Self {
        Self::new([10, 20, 40, 80])
    }
}

impl ExpansionPlan {
    pub fn new(batch_sizes: impl IntoIterator<Item = usize>) -> Self {
        let mut batch_sizes = batch_sizes
            .into_iter()
            .filter(|size| *size > 0)
            .collect::<Vec<_>>();
        batch_sizes.sort_unstable();
        batch_sizes.dedup();
        if batch_sizes.is_empty() {
            batch_sizes.push(1);
        }
        Self { batch_sizes }
    }

    pub fn batches(&self) -> &[usize] {
        &self.batch_sizes
    }

    pub fn next_limit(&self, current: usize, total: usize) -> usize {
        self.batch_sizes
            .iter()
            .copied()
            .find(|size| *size > current)
            .unwrap_or(total)
            .min(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_in_bounded_steps() {
        let plan = ExpansionPlan::default();
        assert_eq!(plan.next_limit(0, 100), 10);
        assert_eq!(plan.next_limit(10, 100), 20);
        assert_eq!(plan.next_limit(80, 100), 100);
    }

    #[test]
    fn normalizes_duplicate_batch_sizes() {
        let plan = ExpansionPlan::new([20, 10, 10, 0]);
        assert_eq!(plan.batches(), &[10, 20]);
    }
}
