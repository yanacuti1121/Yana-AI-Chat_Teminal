// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextBudget {
    pub max_input_tokens: usize,
    pub reserved_output_tokens: usize,
    pub max_files: usize,
    pub max_bytes: usize,
}

impl ContextBudget {
    pub fn new(
        max_input_tokens: usize,
        reserved_output_tokens: usize,
        max_files: usize,
        max_bytes: usize,
    ) -> Self {
        Self {
            max_input_tokens: max_input_tokens.max(1),
            reserved_output_tokens,
            max_files: max_files.max(1),
            max_bytes: max_bytes.max(1),
        }
    }

    pub fn usable_input_tokens(&self) -> usize {
        self.max_input_tokens
            .saturating_sub(self.reserved_output_tokens)
            .max(1)
    }

    pub fn low_memory() -> Self {
        Self::new(4_096, 1_024, 12, 96 * 1024)
    }

    pub fn standard() -> Self {
        Self::new(32_768, 4_096, 48, 768 * 1024)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BudgetUsage {
    pub estimated_tokens: usize,
    pub files: usize,
    pub bytes: usize,
}

impl BudgetUsage {
    pub fn fits(self, budget: ContextBudget) -> bool {
        self.estimated_tokens <= budget.usable_input_tokens()
            && self.files <= budget.max_files
            && self.bytes <= budget.max_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserves_output_capacity() {
        let budget = ContextBudget::new(8_192, 2_048, 20, 100_000);
        assert_eq!(budget.usable_input_tokens(), 6_144);
    }

    #[test]
    fn usage_checks_all_limits() {
        let budget = ContextBudget::new(100, 20, 2, 50);
        assert!(BudgetUsage {
            estimated_tokens: 80,
            files: 2,
            bytes: 50,
        }
        .fits(budget));
        assert!(!BudgetUsage {
            estimated_tokens: 81,
            files: 2,
            bytes: 50,
        }
        .fits(budget));
    }
}
