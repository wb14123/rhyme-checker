use std::fmt;
use std::fmt::Formatter;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum ToneType {
    Ping, // 平声
    Ze, // 仄声
    Zhong, // 平声仄声皆可
    PingYun, // 平声押韵
    ZeYun, // 仄声押韵
    PingYun2, // 平声押韵，换韵
    ZeYun2, // 仄声押韵，换韵
}

/// Will return error if pass in ToneType::Zhong since it can map to either Ping or Ze
pub fn get_basic_tone(t: &ToneType) -> Result<BasicTone> {
    match t {
        ToneType::Ping | ToneType::PingYun | ToneType::PingYun2 => Ok(BasicTone::Ping),
        ToneType::Ze | ToneType::ZeYun | ToneType::ZeYun2 => Ok(BasicTone::Ze),
        _ => bail!("Cannot map to basic tone for type {:?}", t)
    }
}


pub fn tone_match(t1: &ToneType, t2: &ToneType) -> bool {
    if t1 == &ToneType::Zhong || t2 == &ToneType::Zhong {
        return true;
    }
    get_basic_tone(t1).unwrap() == get_basic_tone(t2).unwrap()
}

pub fn tone_is_yun(t: &ToneType) -> bool {
    match t {
        ToneType::PingYun | ToneType::PingYun2 | ToneType::ZeYun | ToneType::ZeYun2 => true,
        _ => false,
    }
}