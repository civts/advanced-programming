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
}
