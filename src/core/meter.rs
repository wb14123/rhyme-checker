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
    println!("{:?}", possible_rhymes);
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
    let mut max_score = 1.0;
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
    let ping: Vec<_> = ping_set.into_iter().collect();
    let ze: Vec<_> = ze_set.into_iter().collect();
    let mut meter_tone_set = HashSet::new();
    for meter_line in meter {
        for meter_tone in meter_line.iter() {
            if meter_tone.rhyme_num.is_some() {
                meter_tone_set.insert(meter_tone.clone());
            }
        }
    }
    println!("{:?}", meter_tone_set);
    get_possible_rhyme_combines(ping, ze, &meter_tone_set)
}

fn get_possible_rhyme_combines(ping_rhymes: Vec<Arc<Rhyme>>, ze_rhymes: Vec<Arc<Rhyme>>,
                               meter_tones_set: &HashSet<MeterTone>) -> Vec<HashMap<MeterTone, Option<Arc<Rhyme>>>> {

    #[derive(Debug)]
    struct State {
        pub ping_idx: usize,
        pub ze_idx: usize,
        pub meter_idx: usize,
        pub rhyme: Option<Arc<Rhyme>>,
        pub prev: Option<Arc<State>>,
    }

    let mut result = vec![];
    let mut states = vec![State{
        ping_idx: 0, ze_idx: 0, meter_idx: 0, rhyme: None, prev: None}];

    let meter_tones: Vec<MeterTone> = meter_tones_set.iter().map(|x| x.clone()).collect();
    while !states.is_empty() {
        let state = Arc::new(states.pop().unwrap());
        if state.meter_idx >= meter_tones.len() {
            let mut backtrace = HashMap::new();
            let mut backtrace_state = Some(state);
            let mut last_meter_idx = 0;
            while backtrace_state.is_some()  {
                let s = backtrace_state.as_ref().unwrap();
                if s.meter_idx > 0 && s.meter_idx != last_meter_idx {
                    backtrace.insert(meter_tones[s.meter_idx-1].clone(), s.rhyme.clone());
                    last_meter_idx = s.meter_idx;
                }
                backtrace_state = s.prev.clone();
            }
            result.push(backtrace);
            continue;
        }
        // TODO: check if ping ze in different group
        if state.ping_idx < ping_rhymes.len() && meter_tones[state.meter_idx].tone == MeterToneType::Ping {
            // put current ping rhyme into the meter position
            states.push(State {
                ping_idx: state.ping_idx + 1,
                ze_idx: state.ze_idx,
                meter_idx: state.meter_idx + 1,
                rhyme: Some(ping_rhymes[state.ping_idx].clone()),
                prev: Some(state.clone()),
            });
            // skip current ping rhyme and try next
            states.push(State {
                ping_idx: state.ping_idx + 1,
                ze_idx: state.ze_idx,
                meter_idx: state.meter_idx,
                rhyme: None,
                prev: Some(state.clone()),
            });
        }
        if state.ze_idx < ze_rhymes.len() && meter_tones[state.meter_idx].tone == MeterToneType::Ze {
            // put current ze rhyme into the meter position
            states.push(State {
                ping_idx: state.ping_idx,
                ze_idx: state.ze_idx + 1,
                meter_idx: state.meter_idx + 1,
                rhyme: Some(ze_rhymes[state.ze_idx].clone()),
                prev: Some(state.clone()),
            });
            // skip current ze rhyme and try next
            states.push(State {
                ping_idx: state.ping_idx,
                ze_idx: state.ze_idx + 1,
                meter_idx: state.meter_idx,
                rhyme: None,
                prev: Some(state.clone())
            })
        }
        // put empty rhyme to the current meter
        states.push(State {
            ping_idx: state.ping_idx,
            ze_idx: state.ze_idx,
            meter_idx: state.meter_idx + 1,
            rhyme: None,
            prev: Some(state.clone()),
        })
    }

    result
}

fn rhyme_in_different_groups(r1: &Option<Arc<Rhyme>>, r2: &Option<Arc<Rhyme>>) -> bool {
    if r1.is_none() || r2.is_none() {
        return false;
    }
    r1.as_ref().unwrap().group != r2.as_ref().unwrap().group
}