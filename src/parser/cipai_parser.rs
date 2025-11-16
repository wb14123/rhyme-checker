use crate::core::meter::CiPai;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Read;
use crate::core::tone::ToneType;

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

fn parse_meter(meter: &str) -> Result<Vec<Vec<ToneType>>> {
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
        .collect::<Result<Vec<Vec<ToneType>>>>()?;
    // remove empty lines at the end
    while result.last().is_some() && result.last().unwrap().len() == 0 {
        result.pop();
    }
    Ok(result)
}

fn parse_meter_line(line: &str) -> Result<Vec<ToneType>> {
    let chars: Vec<char> = line.chars().collect();
    let mut result = vec![];
    let mut i = 0;
    while i < chars.len() {
        let tone = if chars[i] == '－' {
            if i+1 < chars.len() {
                if chars[i+1] == '％' {
                    i += 1;
                    ToneType::PingYun
                } else if chars[i+1] == '＆' {
                    i += 1;
                    ToneType::PingYun2
                } else {
                    ToneType::Ping
                }
            } else {
                ToneType::Ping
            }
        } else if chars[i] == '│' || chars[i] ==  '去' {
            if i + 1 < chars.len() {
                if chars[i+1] == '＊' {
                    i += 1;
                    ToneType::ZeYun
                } else if chars[i+1] ==  '☆' {
                    i += 1;
                    ToneType::ZeYun2
                } else if chars[i+1] ==  '★' { // TODO: 定风波换3次韵
                    i += 1;
                    ToneType::Ze
                } else {
                    ToneType::Ze
                }
            } else {
                ToneType::Ze
            }
        } else if chars[i] == '＋' {
            ToneType::Zhong
        } else {
            bail!("unexpected char in meter: \"{}\", whole sentence: {}", chars[i], line);
        };
        result.push(tone);
        i += 1;
    }
    Ok(result)
}