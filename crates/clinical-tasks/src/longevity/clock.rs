//! Epigenetic clock types — biological age estimation and calibration.

/// Biological age delta (acceleration or deceleration).
///
/// Represents the difference between biological age (estimated from
/// epigenetic clocks) and chronological age.
///
/// # Calibration Warning — SEA Population
///
/// Epigenetic clocks trained on European or North American populations
/// exhibit **unquantified bias** when applied to Southeast Asian (SEA)
/// populations. Clocks must be calibrated and validated on SEA cohorts
/// before quantitative use. Using uncalibrated clocks as biomarkers
/// without acknowledging this limitation is a protocol violation.
///
/// Always check `calibration_status.is_validated()` before reporting
/// quantitative age acceleration. Default state is `Uncalibrated`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct BiologicalAgeDelta {
    /// The delta value in years.
    ///
    /// Positive = biological age older than chronological (accelerated aging).
    /// Negative = biological age younger than chronological (decelerated aging).
    pub value: f32,

    /// Clock version used for estimation.
    pub clock_version: ClockVersion,

    /// Calibration and validation status.
    ///
    /// Default is `Uncalibrated`. Must be `Validated` for quantitative use.
    pub calibration_status: CalibrationStatus,

    /// P-value or confidence metric for the delta estimate.
    pub significance: Option<f64>,

    /// Chronological age at measurement (context, not part of delta).
    pub chronological_age: f64,

    /// Biological age estimate (context, not part of delta).
    pub biological_age: f64,
}

#[cfg(feature = "longevity")]
impl BiologicalAgeDelta {
    /// Create a new biological age delta from chronological and biological ages.
    #[must_use]
    pub fn new(chronological: f64, biological: f64) -> Self {
        // SAFETY: f64 to f32 narrowing is intentional. Age delta values are
        // typically in range ±20 years; f32 provides ~7 significant figures,
        // which exceeds clinical reporting precision (usually 0.1-0.01 years).
        Self {
            value: (biological - chronological) as f32,
            chronological_age: chronological,
            biological_age: biological,
            clock_version: ClockVersion::default(),
            calibration_status: CalibrationStatus::default(),
            significance: None,
        }
    }

    /// Set the clock version (fluent API).
    #[must_use]
    pub fn with_clock_version(mut self, version: ClockVersion) -> Self {
        self.clock_version = version;
        self
    }

    /// Set the calibration status (fluent API).
    #[must_use]
    pub fn with_calibration(mut self, calibration: CalibrationStatus) -> Self {
        self.calibration_status = calibration;
        self
    }

    /// Set the significance/p-value (fluent API).
    #[must_use]
    pub fn with_significance(mut self, p_value: f64) -> Self {
        self.significance = Some(p_value);
        self
    }

    /// Check if biological age is accelerated (older than chronological).
    #[must_use]
    pub fn is_accelerated(&self) -> bool {
        self.value > 0.0
    }

    /// Check if biological age is decelerated (younger than chronological).
    #[must_use]
    pub fn is_decelerated(&self) -> bool {
        self.value < 0.0
    }

    /// Check if biological age equals chronological age (no delta).
    ///
    /// This is the third state alongside accelerated/decelerated.
    /// Exact floating-point equality is used as delta is computed from
    /// the same measurement sources; near-zero should also be checked
    /// via `magnitude()` for tolerance-based comparison.
    #[must_use]
    pub fn is_no_delta(&self) -> bool {
        self.value == 0.0
    }

    /// Get the magnitude of age acceleration regardless of direction.
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        self.value.abs()
    }

    /// Check if this delta has been validated for quantitative use.
    #[must_use]
    pub fn is_validated(&self) -> bool {
        self.calibration_status.is_validated()
    }
}

/// Epigenetic clock version/algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "longevity")]
pub enum ClockVersion {
    /// Horvath pan-tissue clock (2013)
    #[default]
    Horvath2013,

    /// Hannum blood clock (2013)
    Hannum2013,

    /// Skin & blood clock (Horvath et al. 2018)
    SkinBlood2018,

    /// `PhenoAge` (Levine et al. 2018) — mortality predictor
    PhenoAge,

    /// `GrimAge` (Lu et al. 2019) — mortality predictor
    GrimAge,

    /// `GrimAge2` (Lu et al. 2022) — updated mortality predictor
    GrimAge2,

    /// `DunedinPACE` (Belsky et al. 2020) - pace of aging
    DunedinPACE2020,

    /// Custom or research clock
    Custom,
}

#[cfg(feature = "longevity")]
impl ClockVersion {
    /// Get a human-readable description of the clock.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Horvath2013 => "Horvath pan-tissue clock (2013)",
            Self::Hannum2013 => "Hannum blood clock (2013)",
            Self::SkinBlood2018 => "Skin & blood clock (2018)",
            Self::PhenoAge => "PhenoAge (Levine 2018)",
            Self::GrimAge => "GrimAge (Lu 2019)",
            Self::GrimAge2 => "GrimAge2 (Lu 2022)",
            Self::DunedinPACE2020 => "DunedinPACE pace of aging (Belsky 2020)",
            Self::Custom => "Custom/research clock",
        }
    }

    /// Check if this clock estimates pace of aging vs. biological age.
    #[must_use]
    pub fn is_pace_clock(&self) -> bool {
        matches!(self, Self::DunedinPACE2020)
    }

    /// Check if this clock is mortality-associated.
    #[must_use]
    pub fn is_mortality_clock(&self) -> bool {
        matches!(self, Self::PhenoAge | Self::GrimAge | Self::GrimAge2)
    }
}

/// Calibration status for biological age estimation.
///
/// Tracks the validation state of clock calibration for a specific
/// population or clinical context. Clocks must be validated before
/// quantitative use as biomarkers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "longevity")]
pub enum CalibrationStatus {
    /// No calibration applied; raw clock output.
    ///
    /// **Warning:** Uncalibrated clocks must not be used as quantitative
    /// biomarkers without acknowledging unquantified bias.
    #[default]
    Uncalibrated,

    /// Calibration pending external validation.
    ///
    /// Initial calibration performed but awaiting independent cohort
    /// validation or peer review.
    PendingValidation,

    /// Calibration validated in an independent cohort.
    ///
    /// The `cohort_n` indicates the size of the validation cohort.
    Validated {
        /// Size of the validation cohort.
        cohort_n: u32,
    },
}

#[cfg(feature = "longevity")]
impl CalibrationStatus {
    /// Check if calibration has been validated.
    #[must_use]
    pub fn is_validated(&self) -> bool {
        matches!(self, Self::Validated { .. })
    }

    /// Check if calibration is pending validation.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::PendingValidation)
    }

    /// Get the validation cohort size if validated.
    #[must_use]
    pub fn cohort_size(&self) -> Option<u32> {
        match self {
            Self::Validated { cohort_n } => Some(*cohort_n),
            _ => None,
        }
    }

    /// Get a human-readable description.
    ///
    /// # Note
    /// Returns `String` (not `&'static str` like `ClockVersion::description()`)
    /// because the `Validated` variant includes the dynamic `cohort_n` value.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Uncalibrated => "Uncalibrated (raw clock output)".to_string(),
            Self::PendingValidation => "Calibration pending validation".to_string(),
            Self::Validated { cohort_n } => format!("Validated (n={cohort_n})"),
        }
    }
}

/// Pace of aging delta (rate acceleration or deceleration).
///
/// Represents the difference between estimated pace of aging and baseline
/// pace for pace-of-aging clocks (e.g., `DunedinPACE`). Unlike biological age
/// clocks, pace clocks produce unitless rate values, not years.
///
/// # When to Use
/// - Use `PaceOfAgeDelta` for pace-of-aging clocks (`is_pace_clock() == true`)
/// - Use `BiologicalAgeDelta` for biological age clocks (years-based deltas)
///
/// # Calibration Warning — SEA Population
///
/// Pace clocks trained on European or North American populations exhibit
/// **unquantified bias** when applied to Southeast Asian (SEA) populations.
/// Clocks must be calibrated and validated on SEA cohorts before quantitative
/// use. Always check `calibration_status.is_validated()` before reporting.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct PaceOfAgeDelta {
    /// The pace rate value (unitless).
    ///
    /// Values > 1.0 indicate accelerated pace of aging.
    /// Values < 1.0 indicate decelerated pace of aging.
    /// Value of 1.0 indicates normal pace.
    pub pace_rate: f32,

    /// Clock version used for estimation.
    pub clock_version: ClockVersion,

    /// Calibration and validation status.
    ///
    /// Default is `Uncalibrated`. Must be `Validated` for quantitative use.
    pub calibration_status: CalibrationStatus,

    /// P-value or confidence metric for the pace estimate.
    pub significance: Option<f64>,

    /// Chronological age at measurement (context).
    pub chronological_age: f64,

    /// Baseline pace reference (context).
    pub baseline_pace: f32,
}

#[cfg(feature = "longevity")]
impl PaceOfAgeDelta {
    /// Create a new pace of aging delta.
    #[must_use]
    pub fn new(pace_rate: f32, chronological_age: f64, baseline_pace: f32) -> Self {
        Self {
            pace_rate,
            chronological_age,
            baseline_pace,
            clock_version: ClockVersion::default(),
            calibration_status: CalibrationStatus::default(),
            significance: None,
        }
    }

    /// Set the clock version (fluent API).
    #[must_use]
    pub fn with_clock_version(mut self, version: ClockVersion) -> Self {
        self.clock_version = version;
        self
    }

    /// Set the calibration status (fluent API).
    #[must_use]
    pub fn with_calibration(mut self, calibration: CalibrationStatus) -> Self {
        self.calibration_status = calibration;
        self
    }

    /// Set the significance/p-value (fluent API).
    #[must_use]
    pub fn with_significance(mut self, p_value: f64) -> Self {
        self.significance = Some(p_value);
        self
    }

    /// Check if pace is accelerated (faster than normal).
    #[must_use]
    pub fn is_accelerated(&self) -> bool {
        self.pace_rate > 1.0
    }

    /// Check if pace is decelerated (slower than normal).
    #[must_use]
    pub fn is_decelerated(&self) -> bool {
        self.pace_rate < 1.0
    }

    /// Check if pace equals baseline (normal pace).
    #[must_use]
    pub fn is_normal_pace(&self) -> bool {
        self.pace_rate == 1.0
    }

    /// Get the magnitude of pace deviation regardless of direction.
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        (self.pace_rate - 1.0).abs()
    }

    /// Check if this delta has been validated for quantitative use.
    #[must_use]
    pub fn is_validated(&self) -> bool {
        self.calibration_status.is_validated()
    }
}
