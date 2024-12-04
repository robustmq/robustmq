use clap::builder::PossibleValue;
use clap::ValueEnum;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum SortType {
    ASC,
    DESC,
}

impl FromStr for SortType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}

impl fmt::Display for SortType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
impl ValueEnum for SortType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::ASC, Self::DESC]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            SortType::ASC => PossibleValue::new("asc"),
            SortType::DESC => PossibleValue::new("desc"),
        })
    }
}
