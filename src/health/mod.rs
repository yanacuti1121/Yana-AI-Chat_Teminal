// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthLevel { Healthy, Degraded, Critical }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSignal {
    pub component: String,
    pub score: u8,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReliabilitySnapshot {
    pub overall_score: u8,
    pub level: HealthLevel,
    pub signals: Vec<HealthSignal>,
}

impl ReliabilitySnapshot {
    pub fn from_signals(mut signals: Vec<HealthSignal>) -> Self {
        for signal in &mut signals { signal.score = signal.score.min(100); }
        signals.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.component.cmp(&b.component)));
        let overall_score = if signals.is_empty() { 100 } else {
            let weighted_sum: u32 = signals.iter().map(|signal| u32::from(signal.score)).sum();
            (weighted_sum / signals.len() as u32) as u8
        };
        let level = match overall_score { 90..=100 => HealthLevel::Healthy, 60..=89 => HealthLevel::Degraded, _ => HealthLevel::Critical };
        Self { overall_score, level, signals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn low_component_score_lowers_snapshot() {
        let snapshot = ReliabilitySnapshot::from_signals(vec![
            HealthSignal { component: "workspace".into(), score: 100, detail: "ok".into() },
            HealthSignal { component: "provider".into(), score: 40, detail: "offline".into() },
        ]);
        assert_eq!(snapshot.overall_score, 70);
        assert_eq!(snapshot.level, HealthLevel::Degraded);
    }
}
