use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use anyhow::{bail, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use colored::control::SHOULD_COLORIZE;
use palette::{FromColor, Hsl, Srgb};

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
pub enum MeterToneType {
    Ping, // 平声
    Ze, // 仄声
    Zhong, // 平声仄声皆可
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct MeterTone {
    pub tone: MeterToneType,
    pub rhyme_num: Option<i32>,
}

impl fmt::Display for MeterTone {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let tone_str = match self.tone {
            MeterToneType::Ping => "平",
            MeterToneType::Ze => "仄",
            MeterToneType::Zhong => "中",
        };

        if SHOULD_COLORIZE.should_colorize() {
            let colored_tone = match self.rhyme_num {
                Some(n) => {
                    let color = get_contrasting_color(n as usize);
                    tone_str.truecolor(color.0, color.1, color.2)
                }
                _ => tone_str.normal(),
            };
            write!(f, "{}", colored_tone)
        } else {
            let anno_tone = match self.rhyme_num {
                Some(num) => format!("{}（韵{}）", tone_str, num),
                _ => tone_str.to_string(),
            };
            write!(f, "{}", anno_tone)
        }
    }
}

pub fn get_contrasting_color(n: usize) -> (u8, u8, u8) {
    // Use golden ratio conjugate for optimal hue distribution
    let golden_ratio_conjugate = 0.618033988749895;
    let hue = (n as f32 * golden_ratio_conjugate) % 1.0;

    // Alternate saturation and lightness for better distinction
    let saturation = if n % 3 == 0 { 0.9 } else if n % 3 == 1 { 1.0 } else { 0.8 };
    let lightness = if n % 2 == 0 { 0.5 } else { 0.65 };

    let hsl = Hsl::new(hue * 360.0, saturation, lightness);
    let rgb: Srgb = Srgb::from_color(hsl);

    (
        (rgb.red * 255.0) as u8,
        (rgb.green * 255.0) as u8,
        (rgb.blue * 255.0) as u8,
    )
}


/// Will return error if pass in ToneType::Zhong since it can map to either Ping or Ze
pub fn get_basic_tone(t: &MeterTone) -> Result<BasicTone> {
    match t.tone {
        MeterToneType::Ping  => Ok(BasicTone::Ping),
        MeterToneType::Ze => Ok(BasicTone::Ze),
        _ => bail!("Cannot map to basic tone for type {:?}", t)
    }
}


pub fn tone_match(t1: &BasicTone, t2: &MeterTone) -> bool {
    if t2.tone == MeterToneType::Zhong {
        return true;
    }
    t1 == &get_basic_tone(t2).unwrap()
}

/// 获取格律颜色说明
pub fn get_tone_legend() -> String {
    if SHOULD_COLORIZE.should_colorize() {
        format!(
            "格律说明：平=平声 仄=仄声 中=平仄皆可 {}=平韵 {}=仄韵 {}=换韵后平韵 {}=换韵后仄韵",
            "平(红色)".red(),
            "仄(蓝色)".blue(),
            "平(橙色)".truecolor(255, 165, 0),
            "仄(绿色)".green()
        )
    } else {
        "格律说明：如是韵脚，括号内标注声部".to_string()
    }
}
