use crate::core::cipai::CiPai;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use crate::core::tone::{MeterTone, MeterToneType};

pub fn parse_cipai(file_path: &str) -> Result<Vec<CiPai>> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let result = roxmltree::Document::parse(&content)?
        .descendants()
        .filter(|n| n.has_tag_name("词牌"))
        .flat_map(|n| {
            let names : Vec<String> = n.descendants()
                .filter(|c| c.has_tag_name("名称"))
                .map(|c| c.text().unwrap_or("").into())
                .collect();
            let description = n.descendants()
                .find(|c| c.has_tag_name("说明"))
                .map(|c| get_combined_text(c));
            let result: Result<Vec<CiPai>> = n.descendants()
                .filter(|c| c.has_tag_name("格律"))
                .map(|c| {
                    let meter_str = c.text().context("xml文件中没有找到格律标签")?;
                    let variant = c.attribute("说明").map(|t| t.into());
                    let meter = parse_meter(meter_str)?;
                    Ok(CiPai{
                        names: names.clone(),
                        variant,
                        description: description.clone(),
                        meter
                    })
                })
                .collect();
            if result.is_err() {
                eprintln!("Error to parse CiPai xml: {:?}", result.as_ref().err().unwrap())
            }
            result.unwrap_or(vec![])
        })
        .collect();
    Ok(result)
}

fn get_combined_text(node: roxmltree::Node) -> String {
    let mut text = String::new();
    // Recursively collect all text nodes, ignoring element tags
    for child in node.descendants() {
        if child.is_text() {
            text.push_str(child.text().unwrap_or(""));
        }
    }
    text.trim().into()
}

fn parse_meter(meter: &str) -> Result<Vec<Vec<MeterTone>>> {
    let delimiters = vec!['。',  '，',  '、', '\n'];
    let mut meter_str = meter
        // 忽略对偶句
        .replace("{", "")
        .replace("｛", "")
        .replace("}", "")
        .replace("｝", "")
        // 忽略衬字
        .replace("ˇ", "")
        // 忽略领格字
        .replace("～", "")
        .replace("！", "")
        // 忽略领叠韵句
        .replace("[", "")
        .replace("［", "")
        .replace("]", "")
        .replace("］", "")
        // 忽略可选可省略
        .replace("（", "")
        .replace("）", "")
        // 忽略可选增韵
        .replace("＃", "")
        // remove spaces
        .replace(" ", "");
    for d in &delimiters {
        let pattern = format!("{}\n", d);
        meter_str = meter_str.replace(&pattern, &d.to_string());
    }
    let mut result = meter_str.split(|c| delimiters.contains(&c))
        .map(parse_meter_line)
        .collect::<Result<Vec<Vec<MeterTone>>>>()?;
    // remove empty lines at the end
    while result.last().is_some() && result.last().unwrap().len() == 0 {
        result.pop();
    }
    Ok(result)
}

fn parse_meter_line(line: &str) -> Result<Vec<MeterTone>> {
    let chars: Vec<char> = line.chars().collect();
    let mut result = vec![];
    let mut i = 0;
    while i < chars.len() {
        let tone = if chars[i] == '－' {
            if i+1 < chars.len() {
                if chars[i+1] == '％' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ping,
                        rhyme_num: Some(0),
                    }
                } else if chars[i+1] == '＆' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ping,
                        rhyme_num: Some(1),
                    }
                } else if chars[i+1] >= 'a' && chars[i+1] <= 'z' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ping,
                        rhyme_num: Some((chars[i] as i32)-('a' as i32)+1),
                    }
                } else {
                    MeterTone {
                        tone: MeterToneType::Ping,
                        rhyme_num: None,
                    }
                }
            } else {
                MeterTone {
                    tone: MeterToneType::Ping,
                    rhyme_num: None,
                }
            }
        } else if chars[i] == '│' || chars[i] ==  '去' {
            if i + 1 < chars.len() {
                if chars[i+1] == '＊' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ze,
                        rhyme_num: Some(0),
                    }
                } else if chars[i+1] ==  '☆' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ze,
                        rhyme_num: Some(1),
                    }
                } else if chars[i+1] ==  '★' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ze,
                        rhyme_num: Some(2),
                    }
                } else if chars[i+1] >= 'a' && chars[i+1] <= 'z' {
                    i += 1;
                    MeterTone {
                        tone: MeterToneType::Ze,
                        rhyme_num: Some((chars[i] as i32)-('a' as i32)+1),
                    }
                } else {
                    MeterTone {
                        tone: MeterToneType::Ze,
                        rhyme_num: None,
                    }
                }
            } else {
                MeterTone {
                    tone: MeterToneType::Ze,
                    rhyme_num: None,
                }
            }
        } else if chars[i] == '＋' {
            MeterTone {
                tone: MeterToneType::Zhong,
                rhyme_num: None,
            }
        } else {
            bail!("unexpected char in meter: \"{}\", whole sentence: {}", chars[i], line);
        };
        result.push(tone);
        i += 1;
    }
    Ok(result)
}