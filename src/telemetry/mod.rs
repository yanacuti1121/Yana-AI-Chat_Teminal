// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSample {
    pub name: String,
    pub value: u64,
    pub unit: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub struct LocalTelemetry {
    capacity: usize,
    samples: VecDeque<MetricSample>,
}

impl LocalTelemetry {
    pub fn with_capacity(capacity: usize) -> Self { Self { capacity: capacity.max(1), samples: VecDeque::new() } }
    pub fn record(&mut self, sample: MetricSample) {
        self.samples.push_back(sample);
        while self.samples.len() > self.capacity { self.samples.pop_front(); }
    }
    pub fn latest(&self, name: &str) -> Option<&MetricSample> { self.samples.iter().rev().find(|sample| sample.name == name) }
    pub fn samples(&self) -> impl Iterator<Item = &MetricSample> { self.samples.iter() }
}

impl Default for LocalTelemetry { fn default() -> Self { Self::with_capacity(512) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn telemetry_is_bounded() {
        let mut telemetry = LocalTelemetry::with_capacity(2);
        for value in 0..3 { telemetry.record(MetricSample { name: "latency".into(), value, unit: "ms".into(), timestamp_ms: value }); }
        assert_eq!(telemetry.samples().count(), 2);
        assert_eq!(telemetry.latest("latency").unwrap().value, 2);
    }
}
