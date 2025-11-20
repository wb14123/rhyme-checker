use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::core::tone::MeterTone;

// 词牌
#[derive(Serialize, Deserialize)]
pub struct CiPai {
    pub names: Vec<String>,
    pub variant: Option<String>,
    pub description: Option<String>,
    pub meter: Vec<Vec<MeterTone>>,
}

impl Display for CiPai {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "词牌名：{}", self.names[0])?;

        if self.names.len() > 1 {
            writeln!(f, "别名：{}", self.names[1..].join("、"))?;
        }

        if let Some(variant) = &self.variant {
            writeln!(f, "变体：{}", variant)?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "说明：{}", description)?;
        }

        write!(f, "格律：")?;
        for line in &self.meter {
            write!(f, "\n--- ")?;
            for tone in line {
                write!(f, "{}", tone)?;
            }
        }

        Ok(())
    }
}

