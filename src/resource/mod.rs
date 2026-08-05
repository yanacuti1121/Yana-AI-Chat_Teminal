// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accelerator {
    Cpu,
    Metal,
    Cuda,
    Vulkan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub vram_mb: u64,
    pub accelerator: Accelerator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequirement {
    pub model_id: String,
    pub minimum_ram_mb: u64,
    pub preferred_ram_mb: u64,
    pub minimum_vram_mb: u64,
    pub context_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    Comfortable,
    Tight,
    Insufficient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePlan {
    pub fit: Fit,
    pub use_accelerator: bool,
    pub suggested_context_tokens: u32,
    pub reason: String,
}

pub fn plan(snapshot: &ResourceSnapshot, requirement: &ModelRequirement) -> ResourcePlan {
    if snapshot.available_ram_mb < requirement.minimum_ram_mb
        || snapshot.vram_mb < requirement.minimum_vram_mb
    {
        return ResourcePlan {
            fit: Fit::Insufficient,
            use_accelerator: false,
            suggested_context_tokens: requirement.context_tokens.min(2_048),
            reason: "available memory is below the model minimum".into(),
        };
    }

    let comfortable = snapshot.available_ram_mb >= requirement.preferred_ram_mb;
    let use_accelerator = snapshot.accelerator != Accelerator::Cpu && snapshot.vram_mb > 0;
    ResourcePlan {
        fit: if comfortable { Fit::Comfortable } else { Fit::Tight },
        use_accelerator,
        suggested_context_tokens: if comfortable {
            requirement.context_tokens
        } else {
            requirement.context_tokens.min(4_096)
        },
        reason: if comfortable {
            "model fits with working headroom".into()
        } else {
            "model fits, but context should be constrained".into()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrains_context_for_tight_local_fit() {
        let result = plan(
            &ResourceSnapshot {
                total_ram_mb: 8_192,
                available_ram_mb: 6_000,
                vram_mb: 0,
                accelerator: Accelerator::Metal,
            },
            &ModelRequirement {
                model_id: "compressed-local".into(),
                minimum_ram_mb: 4_000,
                preferred_ram_mb: 8_000,
                minimum_vram_mb: 0,
                context_tokens: 16_384,
            },
        );
        assert_eq!(result.fit, Fit::Tight);
        assert_eq!(result.suggested_context_tokens, 4_096);
    }
}
