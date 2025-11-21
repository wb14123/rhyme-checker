use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::core::tone::MeterTone;
use crate::core::meter::{match_meter, MeterMatchResult};
use crate::core::rhyme::RhymeDict;

// 词牌
#[derive(Serialize, Deserialize)]
pub struct CiPai {
    pub names: Vec<String>,
    pub variant: Option<String>,
    pub description: Option<String>,
    pub meter: Vec<Vec<MeterTone>>,
}

impl CiPai {
    pub fn get_max_rhyme_num(&self) -> i32 {
        self.meter
            .iter()
            .flat_map(|line| line.iter())
            .filter_map(|tone| tone.rhyme_num)
            .max()
            .unwrap_or(0)
    }
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

pub struct CiPaiMatchResult<'a> {
    pub cipai: &'a CiPai,
    pub match_result: MeterMatchResult,
}

impl Display for CiPaiMatchResult<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "词牌名：{}", self.cipai.names[0])?;
        if self.cipai.names.len() > 1 {
            writeln!(f, "别名：{}", self.cipai.names[1..].join("、"))?;
        }
        write!(f, "{}", self.match_result)
    }
}

/// Find the best matching CiPai from a list of candidates
///
/// Takes a vector of CiPai, a rhyme dictionary, and input text.
/// Returns a vector of CiPaiMatchResult sorted by score in descending order.
pub fn best_match<'a>(
    cipais: &'a [CiPai],
    rhyme_dict: &RhymeDict,
    input_text: &str,
) -> Vec<CiPaiMatchResult<'a>> {
    let mut results: Vec<CiPaiMatchResult> = cipais
        .iter()
        .map(|cipai| {
            // Convert Vec<Vec<MeterTone>> to Vec<Arc<[MeterTone]>>
            let meter: Vec<std::sync::Arc<[MeterTone]>> = cipai
                .meter
                .iter()
                .map(|line| std::sync::Arc::from(line.as_slice()))
                .collect();

            let match_result = match_meter(rhyme_dict, input_text, &meter, true);

            CiPaiMatchResult {
                cipai,
                match_result,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        a.match_result
            .partial_cmp(&b.match_result)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

