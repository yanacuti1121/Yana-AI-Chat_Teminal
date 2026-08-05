// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::model::ModelError;

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn check(&self) -> Result<(), ModelError> {
        if self.is_cancelled() {
            Err(ModelError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl RetryPolicy {
    pub fn new(max_attempts: u8, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            initial_backoff_ms: initial_backoff_ms.max(1),
            max_backoff_ms: max_backoff_ms.max(initial_backoff_ms.max(1)),
        }
    }

    pub fn should_retry(&self, attempt: u8, error: &ModelError) -> bool {
        attempt < self.max_attempts && error.retryable()
    }

    pub fn backoff_ms(&self, attempt: u8) -> u64 {
        let exponent = attempt.saturating_sub(1).min(16) as u32;
        self.initial_backoff_ms
            .saturating_mul(2_u64.saturating_pow(exponent))
            .min(self.max_backoff_ms)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, 250, 4_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_shared_between_clones() {
        let token = CancellationToken::default();
        let worker = token.clone();
        token.cancel();
        assert!(worker.check().is_err());
    }

    #[test]
    fn retry_only_applies_to_transient_errors() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(1, &ModelError::Timeout));
        assert!(!policy.should_retry(1, &ModelError::Authentication));
        assert!(!policy.should_retry(3, &ModelError::Timeout));
    }

    #[test]
    fn backoff_is_capped() {
        let policy = RetryPolicy::new(8, 100, 500);
        assert_eq!(policy.backoff_ms(1), 100);
        assert_eq!(policy.backoff_ms(4), 500);
    }
}
