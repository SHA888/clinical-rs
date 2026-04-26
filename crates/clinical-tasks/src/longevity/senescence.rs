//! Senescence markers — cellular aging indicators following critical illness.

/// Senescence-associated secretory phenotype (SASP) composite score.
///
/// This represents a weighted combination of pro-inflammatory cytokines
/// and matrix metalloproteinases typically elevated in senescent cells.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct SaspComposite {
    /// The composite score value (typically 0.0 to 1.0, higher = more senescent)
    pub score: f64,

    /// Number of individual markers included in the composite
    pub num_markers: usize,

    /// Confidence interval (lower bound)
    pub ci_lower: Option<f64>,

    /// Confidence interval (upper bound)
    pub ci_upper: Option<f64>,
}

#[cfg(feature = "longevity")]
impl SaspComposite {
    /// Create a new SASP composite with the given score and marker count.
    #[must_use]
    pub fn new(score: f64, num_markers: usize) -> Self {
        Self {
            score,
            num_markers,
            ci_lower: None,
            ci_upper: None,
        }
    }

    /// Compute a weighted SASP composite from raw biomarker concentrations.
    ///
    /// Weights are derived from their relative effect sizes in aging literature
    /// (López-Otín et al., 2023, *Cell* "Hallmarks of Aging"):
    /// - IL-6:   0.35 (strongest SASP cytokine / mortality predictor)
    /// - GDF-15: 0.30 (robust aging biomarker across tissues)
    /// - IL-8:   0.20 (SASP cytokine, co-expressed with IL-6)
    /// - MMP-3:  0.15 (tissue-remodeling, weaker aging signal)
    ///
    /// Each component is z-score-normalised using published reference ranges
    /// before weighting. Returns `None` if fewer than 2 of the 4 components
    /// are present.
    ///
    /// # Reference ranges used for normalisation
    /// - IL-6:   mean 1.5 pg/mL, sd 1.2 pg/mL (healthy adults)
    /// - GDF-15: mean 700 pg/mL, sd 300 pg/mL (healthy adults)
    /// - IL-8:   mean 4.0 pg/mL, sd 3.0 pg/mL (healthy adults)
    /// - MMP-3:  mean 5.0 ng/mL, sd 2.5 ng/mL (healthy adults)
    #[must_use]
    pub fn compute(
        il6_pgml: Option<f32>,
        il8_pgml: Option<f32>,
        gdf15_pgml: Option<f32>,
        mmp3_ngml: Option<f32>,
    ) -> Option<Self> {
        // Reference means and standard deviations for z-score normalisation
        const IL6_MEAN: f64 = 1.5;
        const IL6_SD: f64 = 1.2;
        const GDF15_MEAN: f64 = 700.0;
        const GDF15_SD: f64 = 300.0;
        const IL8_MEAN: f64 = 4.0;
        const IL8_SD: f64 = 3.0;
        const MMP3_MEAN: f64 = 5.0;
        const MMP3_SD: f64 = 2.5;

        // Weights (must sum to 1.0)
        const W_IL6: f64 = 0.35;
        const W_GDF15: f64 = 0.30;
        const W_IL8: f64 = 0.20;
        const W_MMP3: f64 = 0.15;

        let mut weighted_sum = 0.0_f64;
        let mut weight_total = 0.0_f64;
        let mut num_markers = 0_usize;

        if let Some(v) = il6_pgml {
            let z = (f64::from(v) - IL6_MEAN) / IL6_SD;
            weighted_sum += W_IL6 * z;
            weight_total += W_IL6;
            num_markers += 1;
        }
        if let Some(v) = gdf15_pgml {
            let z = (f64::from(v) - GDF15_MEAN) / GDF15_SD;
            weighted_sum += W_GDF15 * z;
            weight_total += W_GDF15;
            num_markers += 1;
        }
        if let Some(v) = il8_pgml {
            let z = (f64::from(v) - IL8_MEAN) / IL8_SD;
            weighted_sum += W_IL8 * z;
            weight_total += W_IL8;
            num_markers += 1;
        }
        if let Some(v) = mmp3_ngml {
            let z = (f64::from(v) - MMP3_MEAN) / MMP3_SD;
            weighted_sum += W_MMP3 * z;
            weight_total += W_MMP3;
            num_markers += 1;
        }

        if num_markers < 2 {
            return None;
        }

        // Rescale by total weight present so partial composites remain on comparable scale
        let score = weighted_sum / weight_total;

        Some(Self {
            score,
            num_markers,
            ci_lower: None,
            ci_upper: None,
        })
    }

    /// Set confidence intervals (fluent API).
    ///
    /// # Panics
    /// Does not panic, but callers should ensure `lower <= upper`.
    #[must_use]
    pub fn with_confidence_interval(mut self, lower: f64, upper: f64) -> Self {
        debug_assert!(lower <= upper, "ci_lower must be <= ci_upper");
        self.ci_lower = Some(lower);
        self.ci_upper = Some(upper);
        self
    }
}

/// Cellular senescence burden score.
///
/// Represents the estimated proportion of senescent cells in relevant
/// tissue samples or the overall senescence burden in the patient.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct SenescenceScore {
    /// The burden score (0.0 to 1.0, where 1.0 = 100% senescent)
    pub burden: f64,

    /// Tissue type if known (e.g., "blood", "skin", "muscle")
    pub tissue_type: Option<String>,

    /// Measurement method used
    pub method: Option<SenescenceMethod>,

    /// Time since critical illness onset (days)
    pub days_post_illness: Option<u32>,
}

/// Methods for measuring cellular senescence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "longevity")]
pub enum SenescenceMethod {
    /// SA-β-galactosidase staining
    #[default]
    SaBetaGal,

    /// p16INK4a expression measurement
    P16Ink4a,

    /// γ-H2AX foci counting (DNA damage marker)
    GammaH2ax,

    /// Telomere-associated foci (TAF)
    TelomereFoci,

    /// Combined multi-marker approach
    MultiMarker,

    /// Other or unknown method
    Other,
}

#[cfg(feature = "longevity")]
impl SenescenceScore {
    /// Create a new senescence score with the given burden.
    #[must_use]
    pub fn new(burden: f64) -> Self {
        Self {
            burden,
            tissue_type: None,
            method: None,
            days_post_illness: None,
        }
    }

    /// Set the tissue type (fluent API).
    #[must_use]
    pub fn with_tissue(mut self, tissue: impl Into<String>) -> Self {
        self.tissue_type = Some(tissue.into());
        self
    }

    /// Set the measurement method (fluent API).
    #[must_use]
    pub fn with_method(mut self, method: SenescenceMethod) -> Self {
        self.method = Some(method);
        self
    }

    /// Set the days post illness (fluent API).
    #[must_use]
    pub fn with_days_post_illness(mut self, days: u32) -> Self {
        self.days_post_illness = Some(days);
        self
    }
}

/// Functional recovery trajectory following critical illness.
///
/// Categorizes post-ICU functional status based on the PICS (Persistent
/// Inflammation, Immunosuppression, and Catabolism Syndrome) framework.
///
/// # PICS Criteria
/// Post-ICU syndrome is characterized by a constellation of new or worsening
/// impairments after critical illness. See Mira et al. (2017) *Frontiers in
/// Immunology* "Post-Intensive Care Syndrome, Inflammation, and Cognitive
/// Impairment" for clinical characterization of persistent inflammation and
/// functional decline following ICU discharge.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "longevity")]
pub enum FunctionalTrajectory {
    /// Persistent Post-ICU Syndrome (PICS): ongoing inflammation, catabolism,
    /// or functional impairment with failure to recover toward baseline.
    ///
    /// Patient exhibits ≥1 PICS domains:
    /// - Cognitive impairment
    /// - Mental health impairment (depression, anxiety, PTSD)
    /// - Physical impairment (ICU-acquired weakness, impaired mobility)
    Pics {
        /// Days since ICU discharge
        days_post_icu: u32,
        /// Baseline functional status before illness (e.g., 0–100 scale)
        baseline_status: Option<f64>,
        /// Current functional status score (e.g., 0–100 scale)
        current_status: f64,
        /// ADL (Activities of Daily Living) score if available
        adl_score: Option<f64>,
        /// IADL (Instrumental ADL) score if available
        iadl_score: Option<f64>,
        /// 6-minute walk test distance (meters)
        walk_test_distance: Option<f64>,
        /// Hand grip strength (kg)
        grip_strength: Option<f64>,
    },

    /// Actively recovering: showing improvement but not yet at baseline.
    ///
    /// Patient is on an upward trajectory but remains below pre-illness
    /// functional status. Recovery rate indicates pace of improvement.
    Recovering {
        /// Days since ICU discharge
        days_post_icu: u32,
        /// Baseline functional status before illness (e.g., 0–100 scale)
        baseline_status: f64,
        /// Current functional status
        current_status: f64,
        /// Rate of change (status points per week); positive = improving
        recovery_rate: f64,
        /// ADL score if available
        adl_score: Option<f64>,
        /// IADL score if available
        iadl_score: Option<f64>,
        /// 6-minute walk test distance (meters)
        walk_test_distance: Option<f64>,
        /// Hand grip strength (kg)
        grip_strength: Option<f64>,
    },

    /// Recovered: returned to baseline functional status or better.
    ///
    /// Patient has achieved pre-illness functional capacity. May still have
    /// residual symptoms but overall functional status is restored.
    Recovered {
        /// Days since ICU discharge when recovery was achieved
        days_post_icu: u32,
        /// Final functional status at recovery
        final_status: f64,
        /// ADL score at recovery
        adl_score: Option<f64>,
        /// IADL score at recovery
        iadl_score: Option<f64>,
    },
}

#[cfg(feature = "longevity")]
impl FunctionalTrajectory {
    /// Days since ICU discharge.
    ///
    /// All variants track `days_post_icu`, so this never returns `None`.
    #[must_use]
    pub fn days_post_icu(&self) -> u32 {
        match self {
            Self::Pics { days_post_icu, .. }
            | Self::Recovering { days_post_icu, .. }
            | Self::Recovered { days_post_icu, .. } => *days_post_icu,
        }
    }

    /// Functional status score.
    ///
    /// - `Pics` / `Recovering`: returns the **current** status (may change over time)
    /// - `Recovered`: returns the **final** status at time of recovery (static)
    ///
    /// Use `is_recovered()` to determine if this is a final value.
    #[must_use]
    pub fn status(&self) -> f64 {
        match self {
            Self::Pics { current_status, .. } | Self::Recovering { current_status, .. } => {
                *current_status
            }
            Self::Recovered { final_status, .. } => *final_status,
        }
    }

    /// Check if the trajectory indicates active recovery (Recovering variant).
    #[must_use]
    pub fn is_recovering(&self) -> bool {
        matches!(self, Self::Recovering { .. })
    }

    /// Check if the patient has recovered (Recovered variant).
    #[must_use]
    pub fn is_recovered(&self) -> bool {
        matches!(self, Self::Recovered { .. })
    }

    /// Check if the patient is in PICS (Pics variant).
    #[must_use]
    pub fn is_pics(&self) -> bool {
        matches!(self, Self::Pics { .. })
    }

    /// Return a stable, single-word categorical label for Arrow `Utf8` columns.
    ///
    /// Unlike `{:?}`, this returns only the variant name, not embedded numeric fields.
    #[must_use]
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Pics { .. } => "Pics",
            Self::Recovering { .. } => "Recovering",
            Self::Recovered { .. } => "Recovered",
        }
    }

    /// Calculate functional delta from baseline.
    ///
    /// Returns `current_status - baseline_status`. Positive values indicate
    /// improvement above baseline; negative values indicate impairment.
    ///
    /// - `Recovering`: always returns `Some(delta)` (baseline is mandatory)
    /// - `Pics`: returns `Some(delta)` if `baseline_status` is present, else `None`
    /// - `Recovered`: returns `None` (delta already resolved to final status)
    #[must_use]
    pub fn functional_delta(&self) -> Option<f64> {
        match self {
            Self::Recovering {
                baseline_status,
                current_status,
                ..
            } => Some(current_status - baseline_status),
            Self::Pics {
                baseline_status,
                current_status,
                ..
            } => baseline_status.map(|baseline| current_status - baseline),
            Self::Recovered { .. } => None,
        }
    }
}
