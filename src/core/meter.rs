use crate::core::rhyme::{RhymeDict, RhymeId};
use crate::core::tone::ToneType;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub enum MismatchType {
    ToneMismatch, // 平仄不合
    RhymeMismatch, // 韵律不合
    CharMismatch, // 缺字或多字
    Match,
}

// 词牌
struct CiPai {
    name: String,
    alias: Vec<String>,
    description: String,
    meter: [Arc<[ToneType]>],
}

struct ScoreWeight {
    tone: f64, // 平仄
    rhyme: f64, // 韵律
    char: f64, // 缺字或多字
}

/// Calculate the similarity score for two sentences. If length doesn't match, the score is 0.
/// The score should be normalized after.
fn match_sentence(sentence: &str, rule: &[ToneType], rhyme1: RhymeId, rhyme2: RhymeId) -> (f64, Vec<MismatchType>) {
    // TODO: implement this
    (0.0, vec![])
}

#[derive(Clone)]
struct MeterMatchState {
    score: f64,
    match_result: Arc<Vec<MismatchType>>,
    text: Arc<String>,
    meter_idx: usize,
    prev_idx: Option<(usize, usize, usize, usize)>,
}

pub struct SentenceMatchResult {
    pub match_result: Option<Arc<Vec<MismatchType>>>, // is None if either text or meter is None
    pub text: Option<Arc<String>>, // None if there is no meter between the text
    pub meter: Option<Arc<[ToneType]>>,
}


pub struct MeterMatchResult {
    pub score: f64,
    pub result: Vec<SentenceMatchResult>,
}


pub fn match_meter(rhyme_dict: &RhymeDict, text: &[Arc<String>], meter: &[Arc<[ToneType]>]) -> MeterMatchResult {
    let possible_rhymes = get_possible_rhymes(rhyme_dict, text);
    let rhymes_len = possible_rhymes.len();
    let text_len = text.len();
    let meter_len = meter.len();
    let meter_match_len = meter_len * 2 + 1;

    /*
    4d array for storing dynamic programming state for highest possible score.

    The indices are the following:
    1. text_i: The highest score until this line of text
    2. meter_i: Which line of meter rule this text is put.
       If meter_i is even: it's put before the (meter_i)/2 th line
       If meter_i is odd: it's put at the (meter_i)/2 th line
    3. r1_i: Which first rhyme is used
    4. r2_i: Which second rhyme is used
    */
    let mut state: Vec<Vec<Vec<Vec<Option<MeterMatchState>>>>> =
        vec![vec![vec![vec![None; rhymes_len]; rhymes_len]; meter_match_len]; text_len];

    for text_i in 0..text_len {
        for meter_i in 0..meter_match_len {
            for r1_i in 0..rhymes_len {
                for r2_i in 0..rhymes_len {
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
                            &*text[text_i],
                            &*meter_line.unwrap(),
                            possible_rhymes[r1_i],
                            possible_rhymes[r2_i],
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
                        for prev_meter_i in 0..(prev_meter_i_end+1) {
                            // There must be a state for previous sentence, if not, there is a bug
                            let last_match_score= state[pre_text_i][prev_meter_i][r1_i][r2_i].as_ref().unwrap().score;
                            if last_max_score <= last_match_score {
                                last_max_score = last_match_score;
                                last_max_match_idx = Some((pre_text_i, prev_meter_i, r1_i, r2_i));
                            }
                        }
                    }
                    let cur_state = MeterMatchState{
                        score: last_max_score + cur_score,
                        match_result: Arc::new(cur_match),
                        meter_idx: meter_i,
                        text: text[text_i].clone(),
                        prev_idx: last_max_match_idx,
                    };
                    state[text_i][meter_i][r1_i][r2_i] = Some(cur_state);
                }
            }
        }
    }

    // get the max score for the last text line
    let mut max_score = 0.0;
    let mut max_match_idx = None;
    for meter_i in 0..meter_match_len {
        for r1_i in 0..rhymes_len {
            for r2_i in 0..rhymes_len {
                let maybe_state = state[text_len-1][meter_i][r1_i][r2_i].as_ref();
                if maybe_state.is_none() {
                    continue;
                }
                let state = maybe_state.unwrap();
                if max_score <= state.score {
                    max_score = state.score;
                    max_match_idx = Some((text_len-1, meter_i, r1_i, r2_i));
                }
            }
        }
    }
    build_result_form_match_state(state, max_match_idx.unwrap(), meter)
}

fn build_result_form_match_state(state: Vec<Vec<Vec<Vec<Option<MeterMatchState>>>>>, match_idx: (usize, usize, usize, usize), meter: &[Arc<[ToneType]>]) -> MeterMatchResult {
    let mut result = vec![];
    let match_state = state[match_idx.0][match_idx.1][match_idx.2][match_idx.3].as_ref().unwrap();
    let score = match_state.score;
    let mut cur_state = match_state;
    let mut cur_meter_idx = meter.len() - 1;
    while cur_state.prev_idx.is_some() {
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
        let (text_i, meter_i, r1_i, r2_i) = cur_state.prev_idx.unwrap();
        cur_state = state[text_i][meter_i][r1_i][r2_i].as_ref().unwrap();
    }
    result.reverse();
    MeterMatchResult {score, result}
}

fn get_possible_rhymes(rhyme_dict: &RhymeDict, text: &[Arc<String>]) -> Vec<RhymeId> {
    let last_chars: Vec<char> = text.iter()
        .filter_map(|s| s.chars().last()).collect();
    let mut result = HashSet::new();
    for c in last_chars {
        for rhyme in rhyme_dict.get_rhymes_by_char(&c) {
            result.insert(rhyme.id);
        }
    }
    result.into_iter().collect()
}