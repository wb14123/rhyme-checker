use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use anyhow::{bail, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Debug)]
pub enum BasicTone {
    Ping, Ze
}

impl fmt::Display for BasicTone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BasicTone::Ping => write!(f, "平"),
            BasicTone::Ze => write!(f, "仄"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ToneType {
    Ping, // 平声
    Ze, // 仄声
    Zhong, // 平声仄声皆可
    PingYun, // 平声押韵
    ZeYun, // 仄声押韵
    PingYun2, // 平声押韵，换韵
    ZeYun2, // 仄声押韵，换韵
}

impl fmt::Display for ToneType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let tone_str = match self {
            ToneType::Ping => "平",
            ToneType::Ze => "仄",
            ToneType::Zhong => "中",
            ToneType::PingYun => "平",
            ToneType::ZeYun => "仄",
            ToneType::PingYun2 => "平",
            ToneType::ZeYun2 => "仄",
        };

        let colored_tone = match self {
            ToneType::PingYun => tone_str.red(),
            ToneType::ZeYun => tone_str.blue(),
            ToneType::PingYun2 => tone_str.truecolor(255, 165, 0), // orange
            ToneType::ZeYun2 => tone_str.green(),
            _ => tone_str.normal(),
        };

        write!(f, "{}", colored_tone)
    }
}

/// Will return error if pass in ToneType::Zhong since it can map to either Ping or Ze
pub fn get_basic_tone(t: &ToneType) -> Result<BasicTone> {
    match t {
        ToneType::Ping | ToneType::PingYun | ToneType::PingYun2 => Ok(BasicTone::Ping),
        ToneType::Ze | ToneType::ZeYun | ToneType::ZeYun2 => Ok(BasicTone::Ze),
        _ => bail!("Cannot map to basic tone for type {:?}", t)
    }
}


pub fn tone_match(t1: &BasicTone, t2: &ToneType) -> bool {
    if t2 == &ToneType::Zhong {
        return true;
    }
    t1 == &get_basic_tone(t2).unwrap()
}

/// 获取格律颜色说明
pub fn get_tone_legend() -> String {
    format!(
        "格律说明：平=平声 仄=仄声 中=平仄皆可 {}=平韵 {}=仄韵 {}=换韵后平韵 {}=换韵后仄韵",
        "平(红色)".red(),
        "仄(蓝色)".blue(),
        "平(橙色)".truecolor(255, 165, 0),
        "仄(绿色)".green()
    )
}
