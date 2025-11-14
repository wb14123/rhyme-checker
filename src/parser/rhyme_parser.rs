use crate::core::rhyme::{Rhyme, RhymeDict, RhymeId};
use crate::core::tone::BasicTone;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs::File;
use std::sync::Arc;

pub fn parse_pingshui(file_path: &str) -> Result<RhymeDict> {
    let file = File::open(file_path)?;
    let json: Value = serde_json::from_reader(file)?;
    let json_format_err = "平水韵文件格式错误";
    let mut rhymes= vec![];
    let mut rhyme_chars = vec![];
    let mut cur_rhyme_id: RhymeId = 0;
    for (shengbu, chars_map) in json.as_object().context(json_format_err)? {
        let tone = if shengbu == "上平声部" || shengbu == "下平声部" {
            BasicTone::Ping
        } else if shengbu == "上声部" || shengbu == "去声部" || shengbu == "入声部" {
            BasicTone::Ze
        } else {
            bail!("平水韵文件中错误声部名称: {}", shengbu);
        };
        for (name, raw_chars) in chars_map.as_object().context(json_format_err)? {
            // insert rhyme
            let rhyme = Rhyme{
                id: cur_rhyme_id,
                name: name.clone(),
                group: None, // 平水韵为诗韵，平仄不在统一韵部，所以设置为空
                tone: tone.clone(),
            };
            rhymes.push(Arc::new(rhyme));

            // insert chars
            let raw_chars_arr =  raw_chars.as_array().context(json_format_err)?;
            let mut chars = Vec::with_capacity(raw_chars_arr.len());
            for raw_char in raw_chars_arr {
                let str = raw_char.as_str().context(json_format_err)?;
                if str.chars().count() != 1 {
                    bail!("平水韵 Json 文件中不是单字: \"{}\"", str);
                }
                chars.push(str.chars().next().unwrap());
            }
            rhyme_chars.push(chars);

            cur_rhyme_id += 1;
        }
    }
    RhymeDict::new(rhyme_chars, rhymes)
}


pub fn parse_cilin(file_path: &str) -> Result<RhymeDict> {
    let file = File::open(file_path)?;
    let json: Value = serde_json::from_reader(file)?;
    let json_format_err = "词林正韵文件格式错误";
    let mut rhymes= vec![];
    let mut rhyme_chars = vec![];
    let mut cur_rhyme_id: RhymeId = 0;
    for (group, tone_map) in json.as_object().context(json_format_err)? {
        for (tone_str, raw_chars) in tone_map.as_object().context(json_format_err)? {
            let tone = if tone_str == "平声" {
                BasicTone::Ping
            } else if tone_str == "仄声" || tone_str == "入声" { // 不对入声单独处理
                BasicTone::Ze
            } else {
                bail!("词林正韵文件中错误声部名称: {}", tone_str);
            };
            // insert rhyme
            let rhyme = Rhyme {
                id: cur_rhyme_id,
                name: group.clone(),
                group: Some(group.clone()),
                tone: tone.clone(),
            };
            rhymes.push(Arc::new(rhyme));

            // insert chars
            let raw_chars_arr = raw_chars.as_array().context(json_format_err)?;
            let mut chars = Vec::with_capacity(raw_chars_arr.len());
            for raw_char in raw_chars_arr {
                let str = raw_char.as_str().context(json_format_err)?;
                if str.chars().count() != 1 {
                    bail!("词林正韵 Json 文件中不是单字: \"{}\"", str);
                }
                chars.push(str.chars().next().unwrap());
            }
            rhyme_chars.push(chars);

            cur_rhyme_id += 1;
        }
    }
    RhymeDict::new(rhyme_chars, rhymes)
}