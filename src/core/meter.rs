use crate::core::rhyme::{Rhyme, RhymeDict};
use crate::core::tone::{tone_match, BasicTone, ToneType};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct MatchType {
    pub tone_match: bool,
    pub rhyme_match: bool,
}

// 词牌
#[derive(Serialize, Deserialize)]
pub struct CiPai {
    pub names: Vec<String>,
    pub variant: Option<String>,
    pub description: Option<String>,
    pub meter: Vec<Vec<ToneType>>,
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

struct ScoreWeight {
    tone: f64, // 平仄
    rhyme: f64, // 韵律
    char: f64, // 缺字或多字
}

/// Calculate the similarity score for two sentences. If length doesn't match, the score is 0.
/// The score should be normalized after.
/// TODO: check if in the same rhyme group
fn match_sentence(rhyme_dict: &RhymeDict, sentence: &str, rule: &[ToneType],
        ping_yun1: Option<&Rhyme>, ze_yun1: Option<&Rhyme>,
        ping_yun2: Option<&Rhyme>, ze_yun2: Option<&Rhyme>) -> (f64, Vec<MatchType>) {

    let mut result = vec![];
    let mut score = 0.0;
    let chars: Vec<_> = sentence.chars().collect();
    let match_len = max(chars.len(), rule.len());
    for i in 0..match_len {
        if i >= chars.len() || i >= rule.len() {
            result.push(MatchType{tone_match: false, rhyme_match: false});
            continue;
        }
        let rhymes = rhyme_dict.get_rhymes_by_char(&chars[i]);
        let tone_match = rhymes.iter()
            .find(|r| tone_match(&r.tone, &rule[i])).is_some();
        if tone_match {
            score += 0.8;
        }
        let rhyme_match_target = match rule[i] {
            ToneType::PingYun => ping_yun1,
            ToneType::ZeYun => ze_yun1,
            ToneType::PingYun2 => ping_yun2,
            ToneType::ZeYun2 => ze_yun2,
            _ => None,
        };
        let rhyme_match = if rhyme_match_target.is_none() {
            true
        } else {
            rhymes.iter().find(|&r| r.deref() == rhyme_match_target.unwrap()).is_some()
        };
        if rhyme_match {
            score += 0.2;
        }
        result.push(MatchType{tone_match, rhyme_match})
    }
    (score / match_len as f64, result)
}

#[derive(Clone, Debug)]
struct MeterMatchState {
    score: f64,
    match_result: Arc<Vec<MatchType>>,
    text: Arc<String>,
    meter_idx: usize,
    prev_idx: Option<(usize, usize, usize, usize, usize, usize)>,
}

pub struct SentenceMatchResult {
    pub match_result: Option<Arc<Vec<MatchType>>>, // is None if either text or meter is None
    pub text: Option<Arc<String>>, // None if there is no meter between the text
    pub meter: Option<Arc<[ToneType]>>,
}

impl Display for SentenceMatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "+++ ")?;
        if self.text.is_some() {
            for (i, char) in self.text.as_ref().unwrap().chars().enumerate() {
                let char_str = char.to_string();
                let colored_char = if self.match_result.is_none() {
                    char_str.truecolor(200, 200, 200)
                } else {
                    match self.match_result.as_ref().unwrap()[i] {
                        MatchType{tone_match: false, rhyme_match: false} => char_str.red(),
                        MatchType{tone_match: true, rhyme_match: false} =>
                            char_str.truecolor(255, 165, 0), // orange
                        MatchType{tone_match: false, rhyme_match: true} =>
                            char_str.truecolor(255, 165, 210), // pink
                        MatchType{tone_match: true, rhyme_match: true} => char_str.white(),
                    }
                };
                write!(f, "{}", colored_char)?;
            }
        }
        writeln!(f, "")?;
        if self.meter.is_some() {
            write!(f, "--- ")?;
            for tone in self.meter.as_ref().unwrap().as_ref() {
                write!(f, "{}", tone)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}


pub struct MeterMatchResult {
    pub score: f64,
    pub result: Vec<SentenceMatchResult>,
}

impl Display for MeterMatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "匹配分数：{}", self.score)?;
        for r in &self.result {
            write!(f, "{}", r)?;
        }
        Ok(())
    }
}


pub fn match_meter(rhyme_dict: &RhymeDict, text: &[Arc<String>], meter: &[Arc<[ToneType]>]) -> MeterMatchResult {
    let (ping_rhymes, ze_rhymes) = get_possible_rhymes(rhyme_dict, text);
    let mut meter_rhymes = HashSet::new();
    for meter_line in meter {
        let tone = meter_line.last();
        if tone.is_none() {
            continue;
        }
        meter_rhymes.insert(tone.unwrap());
    }

    let ping_yun1_len = if meter_rhymes.contains(&ToneType::PingYun) {
        ping_rhymes.len()
    } else {
        1
    };
    let ze_yun1_len = if meter_rhymes.contains(&ToneType::ZeYun) {
        ze_rhymes.len()
    } else {
        1
    };
    let ping_yun2_len = if meter_rhymes.contains(&ToneType::PingYun2) {
        ping_rhymes.len()
    } else {
        1
    };
    let ze_yun2_len = if meter_rhymes.contains(&ToneType::ZeYun2) {
        ze_rhymes.len()
    } else {
        1
    };

    let text_len = text.len();
    let meter_len = meter.len();
    let meter_match_len = meter_len * 2 + 1;

    let mut state: Vec<Vec<Vec<Vec<Vec<Vec<Option<MeterMatchState>>>>>>> =
        vec![vec![vec![vec![vec![vec![None; ze_yun2_len]; ping_yun2_len]; ze_yun1_len]; ping_yun1_len]; meter_match_len]; text_len];

    for text_i in 0..text_len {
        for meter_i in 0..meter_match_len {
            for ping_yun1_i in 0..ping_yun1_len {
                for ze_yun1_i in 0..ze_yun1_len {
                    for ping_yun2_i in 0..ping_yun2_len {
                        for ze_yun2_i in 0..ze_yun2_len {
                            let meter_line = if meter_i % 2 == 0 {
                                None
                            } else {
                                Some(meter[meter_i / 2].clone())
                            };
                            let (cur_score, cur_match) = if meter_line.is_none() {
                                // This sentence is put between/before/after the rules
                                (0.0, vec![])
                            } else {
                                let (score, result) = match_sentence(
                                    rhyme_dict,
                                    &*text[text_i],
                                    &*meter_line.unwrap(),
                                    ping_rhymes[ping_yun1_i].as_deref(),
                                    ze_rhymes[ze_yun1_i].as_deref(),
                                    ping_rhymes[ping_yun2_i].as_deref(),
                                    ze_rhymes[ze_yun2_i].as_deref(),
                                );
                                (score, result)
                            };
                            let mut last_max_match_idx = None;
                            let mut last_max_score = 0.0;
                            if text_i > 0 {
                                let pre_text_i = text_i - 1;

                                let prev_meter_i_end = if meter_i % 2 == 0 {
                                    /* If the current sentence is put between the rules, the last sentence
                                    can be put at the same position since there can be multiple sentences
                                    in the gap.
                                    */
                                    meter_i
                                } else {
                                    /* if current sentence is put right at a line of the rule, the previous
                                    sentence must be put in a previous position.
                                     */
                                    meter_i - 1
                                };
                                // +1 for exclusive boundary
                                for prev_meter_i in 0..(prev_meter_i_end + 1) {
                                    // There must be a state for previous sentence, if not, there is a bug
                                    let last_match_score =
                                        state[pre_text_i][prev_meter_i][ping_yun1_i][ze_yun1_i][ping_yun2_i][ze_yun2_i]
                                        .as_ref().unwrap().score;
                                    if last_max_score <= last_match_score {
                                        last_max_score = last_match_score;
                                        last_max_match_idx = Some((pre_text_i, prev_meter_i, ping_yun1_i, ze_yun1_i, ping_yun2_i, ze_yun2_i));
                                    }
                                }
                            }
                            let cur_state = MeterMatchState {
                                score: last_max_score + cur_score,
                                match_result: Arc::new(cur_match),
                                meter_idx: meter_i,
                                text: text[text_i].clone(),
                                prev_idx: last_max_match_idx,
                            };
                            state[text_i][meter_i][ping_yun1_i][ze_yun1_i][ping_yun2_i][ze_yun2_i] = Some(cur_state);
                        }
                    }
                }
            }
        }
    }

    // get the max score for the last text line
    let mut max_score = 1.0;
    let mut max_match_idx = None;
    for meter_i in 0..meter_match_len {
        for ping_yun1_i in 0..ping_yun1_len {
            for ze_yun1_i in 0..ze_yun1_len {
                for ping_yun2_i in 0..ping_yun2_len {
                    for ze_yun2_i in 0..ze_yun2_len {
                        let maybe_state = state[text_len - 1][meter_i][ping_yun1_i][ze_yun1_i][ping_yun2_i][ze_yun2_i].as_ref();
                        if maybe_state.is_none() {
                            continue;
                        }
                        let state = maybe_state.unwrap();
                        if max_score <= state.score {
                            max_score = state.score;
                            max_match_idx = Some((text_len - 1, meter_i, ping_yun1_i, ze_yun1_i, ping_yun2_i, ze_yun2_i));
                        }
                    }
                }
            }
        }
    }
    build_result_form_match_state(state, max_match_idx.unwrap(), meter)
}

fn build_result_form_match_state(state: Vec<Vec<Vec<Vec<Vec<Vec<Option<MeterMatchState>>>>>>>,
        match_idx: (usize, usize, usize, usize, usize, usize), meter: &[Arc<[ToneType]>]) -> MeterMatchResult {
    println!("State: {:?}", state);
    let mut result = vec![];
    let match_state = state[match_idx.0][match_idx.1][match_idx.2][match_idx.3][match_idx.4][match_idx.5].as_ref().unwrap();
    let score = match_state.score;
    let mut maybe_cur_state = Some(match_state);
    let mut cur_meter_idx = meter.len() - 1;
    while maybe_cur_state.is_some() {
        let cur_state = maybe_cur_state.unwrap();
        while cur_meter_idx * 2 > cur_state.meter_idx {
            result.push(
                SentenceMatchResult{
                    match_result: None,
                    text: None,
                    meter: Some(meter[cur_meter_idx].clone()),
                }
            );
            cur_meter_idx -= 1;
        }
        let meter_line = if cur_state.meter_idx % 2 == 0 {
            None
        } else {
            Some(meter[cur_state.meter_idx / 2].clone())
        };
        let sentence_match_result = SentenceMatchResult{
            match_result: Some(cur_state.match_result.clone()),
            text: Some(cur_state.text.clone()),
            meter: meter_line,
        };
        result.push(sentence_match_result);
        maybe_cur_state = cur_state.prev_idx.and_then( |prev_idx|
            state[prev_idx.0][prev_idx.1][prev_idx.2][prev_idx.3][prev_idx.4][prev_idx.5].as_ref());
    }
    let mut meter_i = cur_meter_idx as isize;
    meter_i -= 1;
    while meter_i >= 0 {
        result.push(SentenceMatchResult { match_result: None, text: None, meter: Some(meter[meter_i as usize].clone()) });
        meter_i -= 1;
    }
    result.reverse();
    MeterMatchResult {score, result}
}

fn get_possible_rhymes(rhyme_dict: &RhymeDict, text: &[Arc<String>]) -> (Vec<Option<Arc<Rhyme>>>, Vec<Option<Arc<Rhyme>>>) {
    let last_chars: Vec<char> = text.iter()
        .filter_map(|s| s.chars().last()).collect();
    let mut ping_set = HashSet::new();
    let mut ze_set = HashSet::new();
    for c in last_chars {
        for rhyme in rhyme_dict.get_rhymes_by_char(&c) {
            if rhyme.tone == BasicTone::Ping {
                ping_set.insert(rhyme.clone());
            } else {
                ze_set.insert(rhyme.clone());
            }
        }
    }
    let mut ping: Vec<_> = ping_set.into_iter()
        .map(|x| Some(x)).collect();
    ping.insert(0, None);
    let mut ze: Vec<_> = ze_set.into_iter()
        .map(|x| Some(x)).collect();
    ze.insert(0, None);
    (ping, ze)
}