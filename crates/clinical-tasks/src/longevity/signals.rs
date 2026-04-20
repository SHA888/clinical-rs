//! Longevity signals — composite measurements for post-critical-illness biological age.

use crate::longevity::clock::BiologicalAgeDelta;
use crate::longevity::senescence::{FunctionalTrajectory, SaspComposite};

/// Longevity signals collected during and after critical illness.
///
/// All fields are optional to allow for partial data collection.
/// This struct serves as a container for various biological age acceleration
/// markers and senescence-related measurements.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct LongevitySignals {
    /// Biological age delta from epigenetic clock estimation
    pub biological_age_delta: Option<BiologicalAgeDelta>,

    /// Senescence-associated secretory phenotype (SASP) composite score
    pub sasp_composite: Option<SaspComposite>,

    /// Post-ICU functional recovery trajectory
    pub post_icu_functional_trajectory: Option<FunctionalTrajectory>,

    /// GDF-15 concentration (pg/mL) — growth differentiation factor 15, aging biomarker
    pub gdf15_pgml: Option<f32>,

    /// IL-6 concentration (pg/mL) — interleukin 6, pro-inflammatory SASP cytokine
    pub il6_pgml: Option<f32>,

    /// IL-8 concentration (pg/mL) — interleukin 8, pro-inflammatory SASP cytokine
    pub il8_pgml: Option<f32>,

    /// MMP-3 concentration (ng/mL) — matrix metalloproteinase 3, tissue remodeling
    pub mmp3_ngml: Option<f32>,

    /// p16INK4a relative expression — canonical cellular senescence marker
    pub p16_relative_expression: Option<f32>,

    /// Cellular senescence burden scalar (0.0–1.0)
    pub senescence_score: Option<f64>,

    /// Inflammatory age index (e.g., CRP-based)
    pub inflammatory_age: Option<f64>,

    /// Telomere length relative change
    pub telomere_change: Option<f64>,

    /// Mitochondrial dysfunction marker
    pub mitochondrial_marker: Option<f64>,

    /// Oxidative stress indicator
    pub oxidative_stress: Option<f64>,

    /// DNA damage repair capacity
    pub dna_repair_capacity: Option<f64>,

    /// Autophagy flux measurement
    pub autophagy_flux: Option<f64>,
}

#[cfg(feature = "longevity")]
impl LongevitySignals {
    /// Create a new empty `LongevitySignals` instance with all fields as `None`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if at least one signal is present (not `None`).
    #[must_use]
    pub fn has_any_signal(&self) -> bool {
        self.biological_age_delta.is_some()
            || self.sasp_composite.is_some()
            || self.post_icu_functional_trajectory.is_some()
            || self.gdf15_pgml.is_some()
            || self.il6_pgml.is_some()
            || self.il8_pgml.is_some()
            || self.mmp3_ngml.is_some()
            || self.p16_relative_expression.is_some()
            || self.senescence_score.is_some()
            || self.inflammatory_age.is_some()
            || self.telomere_change.is_some()
            || self.mitochondrial_marker.is_some()
            || self.oxidative_stress.is_some()
            || self.dna_repair_capacity.is_some()
            || self.autophagy_flux.is_some()
    }

    /// Returns the number of signals present (not `None`).
    #[must_use]
    pub fn count_present(&self) -> usize {
        [
            self.biological_age_delta.is_some(),
            self.sasp_composite.is_some(),
            self.post_icu_functional_trajectory.is_some(),
            self.gdf15_pgml.is_some(),
            self.il6_pgml.is_some(),
            self.il8_pgml.is_some(),
            self.mmp3_ngml.is_some(),
            self.p16_relative_expression.is_some(),
            self.senescence_score.is_some(),
            self.inflammatory_age.is_some(),
            self.telomere_change.is_some(),
            self.mitochondrial_marker.is_some(),
            self.oxidative_stress.is_some(),
            self.dna_repair_capacity.is_some(),
            self.autophagy_flux.is_some(),
        ]
        .iter()
        .filter(|&&p| p)
        .count()
    }
}
