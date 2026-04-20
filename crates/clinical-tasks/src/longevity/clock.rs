//! Epigenetic clock types — biological age estimation and calibration.

/// Biological age delta (acceleration or deceleration).
///
/// Represents the difference between biological age (estimated from
/// epigenetic clocks) and chronological age.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg(feature = "longevity")]
pub struct BiologicalAgeDelta {
    /// The delta in years (positive = older than chronological, negative = younger)
    pub delta_years: f64,

    /// Chronological age at measurement
    pub chronological_age: f64,

    /// Biological age estimate
    pub biological_age: f64,

    /// Clock version used for estimation
    pub clock_version: ClockVersion,

    /// Calibration status of the clock
    pub calibration: CalibrationStatus,

    /// P-value or confidence metric for the delta estimate
    pub significance: Option<f64>,
}

#[cfg(feature = "longevity")]
impl BiologicalAgeDelta {
    /// Create a new biological age delta from chronological and biological ages.
    #[must_use]
    pub fn new(chronological: f64, biological: f64) -> Self {
        Self {
            delta_years: biological - chronological,
            chronological_age: chronological,
            biological_age: biological,
            clock_version: ClockVersion::default(),
            calibration: CalibrationStatus::default(),
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
        self.calibration = calibration;
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
        self.delta_years > 0.0
    }

    /// Check if biological age is decelerated (younger than chronological).
    #[must_use]
    pub fn is_decelerated(&self) -> bool {
        self.delta_years < 0.0
    }

    /// Get the magnitude of age acceleration regardless of direction.
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        self.delta_years.abs()
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

    /// `PhenoAge` (Levine et al. 2018)
    PhenoAge2018,

    /// `GrimAge` (Lu et al. 2019)
    GrimAge2019,

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
            Self::PhenoAge2018 => "PhenoAge (Levine 2018)",
            Self::GrimAge2019 => "GrimAge (Lu 2019)",
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
        matches!(self, Self::PhenoAge2018 | Self::GrimAge2019)
    }
}

/// Calibration status for biological age estimation.
///
/// Indicates whether the clock has been calibrated for the specific
/// population or clinical context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg(feature = "longevity")]
pub enum CalibrationStatus {
    /// No calibration applied (raw clock output)
    #[default]
    Uncalibrated,

    /// Calibrated for general population
    PopulationCalibrated,

    /// Calibrated for ICU/critical illness population
    IcuCalibrated,

    /// Calibrated for specific disease group
    DiseaseSpecific,

    /// Custom calibration applied
    CustomCalibrated,
}

#[cfg(feature = "longevity")]
impl CalibrationStatus {
    /// Check if any calibration has been applied.
    #[must_use]
    pub fn is_calibrated(&self) -> bool {
        !matches!(self, Self::Uncalibrated)
    }

    /// Get a human-readable description.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Uncalibrated => "Uncalibrated (raw clock output)",
            Self::PopulationCalibrated => "Calibrated for general population",
            Self::IcuCalibrated => "Calibrated for ICU/critical illness",
            Self::DiseaseSpecific => "Calibrated for specific disease group",
            Self::CustomCalibrated => "Custom calibration applied",
        }
    }
}
