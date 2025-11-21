use crate::core::rhyme::{Rhyme, RhymeDict};
use crate::core::tone::{tone_match, BasicTone, MeterTone, MeterToneType};
use colored::control::SHOULD_COLORIZE;
use colored::Colorize;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

/// 获取匹配结果颜色说明
pub fn get_match_legend() -> String {
    if SHOULD_COLORIZE.should_colorize() {
        format!(
            "匹配结果说明：{}=完全匹配 {}=仅音调匹配 {}=不匹配",
            "字(白色)".white(),
            "字(橙色)".truecolor(255, 165, 0),
            "字(红色)".red()
        )
    } else {
        "匹配结果说明：字后括号内说明匹配错误原因：平仄错或是韵脚错".to_string()
    }
}

#[derive(Debug)]
pub enum MatchType {
    NoMatch,
    ToneOnly,
    AllMatch,
}


pub struct SentenceMatchResult {
    pub match_result: Option<Arc<Vec<MatchType>>>, // is None if either text or meter is None
    pub text: Option<Arc<String>>, // None if there is no meter between the text
    pub meter: Option<Arc<[MeterTone]>>,
}

impl Display for SentenceMatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.text.is_some() {
            write!(f, "+++ ")?;
            for (i, char) in self.text.as_ref().unwrap().chars().enumerate() {
                let char_str = char.to_string();
                if SHOULD_COLORIZE.should_colorize() {
                    let colored_char = if self.match_result.is_none() || self.match_result.as_ref().unwrap().len() == 0 {
                        char_str.truecolor(180, 180, 180)
                    } else {
                        match self.match_result.as_ref().unwrap()[i] {
                            MatchType::NoMatch => char_str.red(),
                            MatchType::ToneOnly =>
                                char_str.truecolor(255, 165, 0), // orange
                            MatchType::AllMatch => char_str.white(),
                        }
                    };
                    write!(f, "{}", colored_char)?;
                } else {
                    let anno_char = if self.match_result.is_none() || self.match_result.as_ref().unwrap().len() == 0 {
                        char_str
                    } else {
                        match self.match_result.as_ref().unwrap()[i] {
                            MatchType::NoMatch => format!("{}（平仄错）", char_str),
                            MatchType::ToneOnly => format!("{}（韵脚错）", char_str),
                            MatchType::AllMatch => char_str,
                        }
                    };
                    write!(f, "{}", anno_char)?;
                }
            }
            writeln!(f, "")?;
        }
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

#[derive(Clone, Debug)]
struct MeterMatchState {
    score: f64,
    match_result: Arc<Vec<MatchType>>,
    text: Arc<String>,
    meter_idx: usize,
    prev_idx: Option<(usize, usize, usize)>,
}

pub fn match_meter(rhyme_dict: &RhymeDict, input_text: &str, meter: &[Arc<[MeterTone]>]) -> MeterMatchResult {
    let text = parse_input_text(input_text);
    let possible_rhymes = get_possible_rhymes(rhyme_dict, &text, meter);
    let mut meter_rhymes = HashSet::new();
    for meter_line in meter {
        let tone = meter_line.last();
        if tone.is_none() {
            continue;
        }
        meter_rhymes.insert(tone.unwrap());
    }

    let text_len = text.len();
    let meter_len = meter.len();
    let meter_match_len = meter_len * 2 + 1;
    let rhymes_len = possible_rhymes.len();

    let mut state: Vec<Vec<Vec<Option<MeterMatchState>>>> =
        vec![vec![vec![None; rhymes_len]; meter_match_len]; text_len];

    for text_i in 0..text_len {
        for meter_i in 0..meter_match_len {
            for rhyme_i in 0..rhymes_len {
                let meter_line = if meter_i % 2 == 0 {
                    None
                } else {
                    Some(meter[meter_i / 2].clone())
                };
                let (cur_score, cur_match) =
                    if meter_line.is_none() {
                        // This sentence is put between/before/after the rules
                        (0.0, vec![])
                    } else {
                        let (score, result) = match_sentence(
                            rhyme_dict,
                            &*text[text_i],
                            &*meter_line.unwrap(),
                            &possible_rhymes[rhyme_i],
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
                            state[pre_text_i][prev_meter_i][rhyme_i]
                                .as_ref().unwrap().score;
                        if last_max_score < last_match_score || (last_max_score == last_match_score && last_max_match_idx.is_none()) {
                            last_max_score = last_match_score;
                            last_max_match_idx = Some((pre_text_i, prev_meter_i, rhyme_i));
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
                state[text_i][meter_i][rhyme_i] = Some(cur_state);
            }
        }
    }

    // get the max score for the last text line
    let mut max_score = 0.0;
    let mut max_match_idx = None;
    for meter_i in 0..meter_match_len {
        for rhyme_i in 0..rhymes_len {
            let maybe_state = state[text_len - 1][meter_i][rhyme_i].as_ref();
            if maybe_state.is_none() {
                continue;
            }
            let state = maybe_state.unwrap();
            if max_score < state.score || (max_score == state.score && max_match_idx.is_none()) {
                max_score = state.score;
                max_match_idx = Some((text_len - 1, meter_i, rhyme_i));
            }
        }
    }
    let mut result = build_result_form_match_state(state, max_match_idx.unwrap(), meter);
    let non_empty_meter_len = meter.iter().filter(|m| m.len() > 0).count();
    result.score = result.score / max(text_len, non_empty_meter_len) as f64;
    result
}

fn parse_input_text(text: &str) -> Vec<Arc<String>> {
    let delimiters = vec!['.', '。',  ',', '，', '、', '?', '？', '!',  '！', '\n'];
    text
        .split(|c| delimiters.contains(&c))
        .map(|l| Arc::new(l.trim().to_string()))
        .filter(|l| l.len() > 0)
        .collect()
}


/// Calculate the similarity score for two sentences. If length doesn't match, the score is 0.
/// The score should be normalized after.
fn match_sentence(rhyme_dict: &RhymeDict, sentence: &str, rule: &[MeterTone],
                  rhyme_map: &HashMap<MeterTone, Option<Arc<Rhyme>>>) -> (f64, Vec<MatchType>) {

    let mut result = vec![];
    let mut score = 0.0;
    let chars: Vec<_> = sentence.chars().collect();
    let match_len = max(chars.len(), rule.len());
    for i in 0..match_len {
        if i >= chars.len() || i >= rule.len() {
            result.push(MatchType::NoMatch);
            continue;
        }
        let rhymes = rhyme_dict.get_rhymes_by_char(&chars[i]);
        let tone_match = rhymes.iter()
            .find(|r| tone_match(&r.tone, &rule[i])).is_some();
        if tone_match {
            score += 0.8;
        }
        let (need_count, rhyme_match_target) = if rule[i].rhyme_num.is_none() {
            (false, None)
        } else {
            (true, rhyme_map.get(&rule[i]).unwrap().clone())
        };
        let rhyme_match = if !need_count {
            true
        } else if rhyme_match_target.is_none() {
            false
        } else {
            rhymes.iter().find(|&r| r.deref() == rhyme_match_target.as_ref().unwrap().deref()).is_some()
        };
        if rhyme_match {
            score += 0.2;
        }
        let match_type = if rhyme_match && tone_match {
            MatchType::AllMatch
        } else if tone_match {
            MatchType::ToneOnly
        } else {
            MatchType::NoMatch
        };
        result.push(match_type)
    }
    (score / match_len as f64, result)
}

fn build_result_form_match_state(state: Vec<Vec<Vec<Option<MeterMatchState>>>>,
                                 match_idx: (usize, usize, usize), meter: &[Arc<[MeterTone]>]) -> MeterMatchResult {
    let mut result = vec![];
    let match_state = state[match_idx.0][match_idx.1][match_idx.2].as_ref().unwrap();
    let score = match_state.score;
    let mut maybe_cur_state = Some(match_state);
    let mut cur_meter_idx = (meter.len() - 1) as isize;
    while maybe_cur_state.is_some() {
        let cur_state = maybe_cur_state.unwrap();
        while cur_meter_idx > (cur_state.meter_idx / 2) as isize {
            result.push(
                SentenceMatchResult{
                    match_result: None,
                    text: None,
                    meter: Some(meter[cur_meter_idx as usize].clone()),
                }
            );
            cur_meter_idx -= 1;
        }
        cur_meter_idx -= 1;
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
            state[prev_idx.0][prev_idx.1][prev_idx.2].as_ref());
    }
    while cur_meter_idx >= 0 {
        result.push(SentenceMatchResult { match_result: None, text: None,
            meter: Some(meter[cur_meter_idx as usize].clone()) });
        cur_meter_idx -= 1;
    }
    result.reverse();
    MeterMatchResult {score, result}
}

fn get_possible_rhymes(rhyme_dict: &RhymeDict, text: &Vec<Arc<String>>, meter: &[Arc<[MeterTone]>]
        ) -> Vec<HashMap<MeterTone, Option<Arc<Rhyme>>>> {
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
    let mut meter_tone_set = HashSet::new();
    for meter_line in meter {
        for meter_tone in meter_line.iter() {
            if meter_tone.rhyme_num.is_some() {
                meter_tone_set.insert(meter_tone);
            }
        }
    }

    let mut results = Vec::new();
    let meter_tones: Vec<MeterTone> = meter_tone_set.into_iter().cloned().collect();

    if meter_tones.is_empty() {
        return vec![];
    }

    let mut current = HashMap::new();
    let mut rhyme_num_groups: HashMap<i32, Option<String>> = HashMap::new();

    dfs_rhyme_combines(
        &meter_tones,
        0,
        &mut current,
        &mut rhyme_num_groups,
        &mut ping_set,
        &mut ze_set,
        &mut results,
    );

    results
}

/// DFS to explore all valid rhyme combinations
fn dfs_rhyme_combines(
    meter_tones: &[MeterTone],
    index: usize,
    current: &mut HashMap<MeterTone, Option<Arc<Rhyme>>>,
    rhyme_num_groups: &mut HashMap<i32, Option<String>>,
    ping_rhymes: &mut HashSet<Arc<Rhyme>>,
    ze_rhymes: &mut HashSet<Arc<Rhyme>>,
    results: &mut Vec<HashMap<MeterTone, Option<Arc<Rhyme>>>>,
) {
    // Base case: all meter_tones have been assigned
    if index == meter_tones.len() {
        results.push(current.clone());
        return;
    }

    let meter_tone = &meter_tones[index];

    // Option 1: Assign None to this meter_tone
    current.insert(meter_tone.clone(), None);
    dfs_rhyme_combines(
        meter_tones,
        index + 1,
        current,
        rhyme_num_groups,
        ping_rhymes,
        ze_rhymes,
        results,
    );
    current.remove(meter_tone);

    // Option 2: Assign a rhyme to this meter_tone
    // Clone the set so we can modify the original during iteration
    let available_rhymes = match meter_tone.tone {
        MeterToneType::Ping => ping_rhymes.clone(),
        MeterToneType::Ze => ze_rhymes.clone(),
        MeterToneType::Zhong => panic!("MeterToneType::Zhong should not appear in meter patterns"),
    };

    for rhyme in available_rhymes {
        let rhyme_group = rhyme.group.clone();

        // Validate: if this meter_tone has a rhyme_num with an existing group, they must match
        let rhyme_num = meter_tone.rhyme_num.unwrap();
        if !rhyme_num_groups.contains_key(&rhyme_num) {
            rhyme_num_groups.insert(rhyme_num, rhyme_group.clone());
        }
        if rhyme_num_groups.get(&rhyme_num).unwrap() != &rhyme_group {
            continue;
        }
        current.insert(meter_tone.clone(), Some(rhyme.clone()));

        // Remove rhyme from the appropriate set
        let from_ping = ping_rhymes.remove(&rhyme);
        if !from_ping {
            ze_rhymes.remove(&rhyme);
        }

        dfs_rhyme_combines(
            meter_tones,
            index + 1,
            current,
            rhyme_num_groups,
            ping_rhymes,
            ze_rhymes,
            results,
        );

        // clean up state after recursive call
        if from_ping {
            ping_rhymes.insert(rhyme.clone());
        } else {
            ze_rhymes.insert(rhyme.clone());
        }
        rhyme_num_groups.remove(&rhyme_num);
        current.remove(meter_tone);
    }
}