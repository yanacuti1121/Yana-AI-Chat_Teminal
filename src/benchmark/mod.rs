// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkSample {
    pub elapsed_micros: u64,
    pub items: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkSummary {
    pub samples: usize,
    pub min_micros: u64,
    pub max_micros: u64,
    pub median_micros: u64,
    pub total_items: u64,
}

pub fn summarize(samples: &[BenchmarkSample]) -> Option<BenchmarkSummary> {
    if samples.is_empty() {
        return None;
    }
    let mut elapsed = samples
        .iter()
        .map(|sample| sample.elapsed_micros)
        .collect::<Vec<_>>();
    elapsed.sort_unstable();
    Some(BenchmarkSummary {
        samples: elapsed.len(),
        min_micros: elapsed[0],
        max_micros: elapsed[elapsed.len() - 1],
        median_micros: elapsed[elapsed.len() / 2],
        total_items: samples.iter().map(|sample| sample.items).sum(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_deterministically() {
        let result = summarize(&[
            BenchmarkSample { elapsed_micros: 30, items: 3 },
            BenchmarkSample { elapsed_micros: 10, items: 1 },
            BenchmarkSample { elapsed_micros: 20, items: 2 },
        ])
        .unwrap();
        assert_eq!(result.min_micros, 10);
        assert_eq!(result.median_micros, 20);
        assert_eq!(result.max_micros, 30);
        assert_eq!(result.total_items, 6);
    }
}
