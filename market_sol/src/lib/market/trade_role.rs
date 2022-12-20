/// Trade role (Importer / Exporter) with needs (in EUR)
///
/// Positive needs -> Need to import
///
/// Negative needs -> Able to export
#[derive(PartialEq, Clone, Debug, Copy)]
pub(crate) enum TradeRole {
    Importer { need: f32 },
    Exporter { need: f32 },
}

impl TradeRole {
    /// Increase need by n
    ///
    /// n should be expressed in DEFAULT_GOOD_VALUE
    pub(crate) fn increase_need(&mut self, n: f32) {
        match self {
            TradeRole::Importer { need, .. } => *need += n,
            TradeRole::Exporter { need, .. } => *need += n,
        }
    }

    /// Decrease need by n
    ///
    /// n should be expressed in DEFAULT_GOOD_VALUE
    pub(crate) fn decrease_need(&mut self, n: f32) {
        match self {
            TradeRole::Importer { need, .. } => *need -= n,
            TradeRole::Exporter { need, .. } => *need -= n,
        }
    }

    /// Switch to another role if the need does not match its role anymore
    ///
    /// Example:
    ///
    /// Importer with need < 0 -> Exporter
    ///
    /// Exporter with need > 0 -> Importer
    pub(crate) fn switch(&self) -> Self {
        match self {
            TradeRole::Importer { need } if need < &0f32 => TradeRole::Exporter { need: *need },
            TradeRole::Exporter { need } if need > &0f32 => TradeRole::Importer { need: *need },
            _ => self.clone(),
        }
    }
}
