#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum StrategyName {
    Stocastic,
    Quantity,
    Others,
}

pub(crate) const ALL_STRATEGY_NAMES: [StrategyName; 3] = [
    StrategyName::Others,
    StrategyName::Quantity,
    StrategyName::Stocastic,
];

impl ToString for StrategyName {
    fn to_string(&self) -> String {
        String::from(match self {
            StrategyName::Stocastic => "Stocastic",
            StrategyName::Quantity => "Quantity",
            StrategyName::Others => "Others",
        })
    }
}
