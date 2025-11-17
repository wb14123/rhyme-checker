mod core;
mod parser;

use std::sync::Arc;
use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use parser::rhyme_parser::parse_pingshui;
use parser::cipai_parser::parse_cipai;
use crate::core::meter::{get_match_legend, match_meter};
use crate::core::rhyme::RhymeDict;
use crate::core::tone::{ToneType, get_tone_legend};
use crate::parser::rhyme_parser::parse_cilin;

#[derive(Debug, Clone, ValueEnum)]
enum DictType {
    /// 平水韵
    Pingshui,
    /// 词林正韵
    Cilin,
}

#[derive(Parser)]
#[command(name = "rhyme-checker")]
#[command(about = "查询汉字韵律信息", long_about = None)]
struct Cli {
    /// 数据文件夹路径
    #[arg(short, long, default_value = "data")]
    data_dir: String,

    /// 韵书类型
    #[arg(short = 't', long, value_enum, default_value = "pingshui")]
    dict_type: DictType,

    /// 输出不用颜色区分格律和结果
    #[arg(long)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查询汉字的韵部信息
    QueryCharRhyme {
        /// 要查询的汉字
        #[arg(value_name = "CHAR")]
        character: String,

        /// 显示该韵部的所有汉字
        #[arg(short, long)]
        show_all: bool,
    },

    /// 查询词牌信息
    QueryCiPai {
        /// 要查询的词牌名
        #[arg(short, long)]
        ci_pai: String,

        /// 格律变种，如定格、格一等，可选，如为空则显示此词牌所有格律变种
        #[arg(short, long)]
        variant: Option<String>,
    },

    /// 检查格律
    MatchCiPai {
        /// 词牌名
        #[arg(short, long)]
        ci_pai: String,

        /// 格律变种，如定格、格一等
        #[arg(short, long)]
        variant: String,

        #[arg(value_name = "TEXT")]
        text: String,
    }
}

fn query_char_rhyme(rhyme_dict: &RhymeDict, character: &str, show_all: bool) -> Result<()> {

    if character.chars().count() != 1 {
        bail!("请输入单个汉字");
    }

    let query_char = character.chars().next().unwrap();

    // Query rhyme information
    let rhymes = rhyme_dict.get_rhymes_by_char(&query_char);

    if rhymes.is_empty() {
        println!("未找到韵律信息: {}", query_char);
        return Ok(());
    }

    for rhyme in rhymes {
        println!("韵部: {}", rhyme);

        if show_all {
            let chars = rhyme_dict.get_chars_by_rhyme(&rhyme.id);
            println!("该韵部的所有字 ({} 个):", chars.len());

            // Display characters in rows of 20 for better readability
            for (i, c) in chars.iter().enumerate() {
                print!("{}", c);
                if (i + 1) % 20 == 0 {
                    println!();
                } else {
                    print!(" ");
                }
            }
            if chars.len() % 20 != 0 {
                println!();
            }
        }
    }

    Ok(())
}

fn query_cipai(file: &str, name: &str, variant: Option<&String>) -> Result<()> {
    let cipai_list = parse_cipai(file)?;

    let matching_cipai: Vec<_> = cipai_list
        .iter()
        .filter(|cipai| {
            let cipai_match = cipai.names.iter().any(|n| n.contains(name));
            let variant_match = variant.is_none() || (variant == cipai.variant.as_ref());
            cipai_match && variant_match
        })
        .collect();

    if matching_cipai.is_empty() {
        println!("未找到词牌: {}", name);
        return Ok(());
    }

    println!("{}\n", get_tone_legend());

    for (i, cipai) in matching_cipai.iter().enumerate() {
        if i > 0 {
            println!("\n{}", "=".repeat(60));
        }
        print!("{}", cipai);
    }
    println!();

    Ok(())
}

fn match_cipai(rhyme_dict: &RhymeDict, file: &str, name: &str, variant: &str, text: &str) -> Result<()> {

    let cipai_list = parse_cipai(file)?;
    let cipai= cipai_list
        .iter()
        .find(|cipai| cipai.names.iter().any(|n| n.contains(name))
            && cipai.variant.is_some()
            && cipai.variant.as_ref().unwrap() == variant
        );
    if cipai.is_none() {
        bail!("未找到词牌: {}, {}", name, variant);
    }
    let meter_vec: Vec<Arc<[ToneType]>> = cipai.as_ref().unwrap().meter.iter()
        .cloned()
        .map(Into::into)
        .collect();
    println!("{}", get_tone_legend());
    println!("{}\n", get_match_legend());
    let result = match_meter(rhyme_dict, text, &meter_vec);
    println!("{}", result);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let rhyme_dict = match cli.dict_type {
        DictType::Pingshui => parse_pingshui(format!("{}/rhyme/Pingshui_Rhyme.json", cli.data_dir).as_str())?,
        DictType::Cilin => parse_cilin(format!("{}/rhyme/Cilin_Rhyme.json", cli.data_dir).as_str())?,
    };

    let cipai_file = format!("{}/cipai/cipai.xml", cli.data_dir);

    match &cli.command {
        Commands::QueryCharRhyme { character, show_all} =>
            query_char_rhyme(&rhyme_dict, character, *show_all)?,
        Commands::QueryCiPai { ci_pai, variant } =>
            query_cipai(cipai_file.as_str(), ci_pai, variant.as_ref())?,
        Commands::MatchCiPai {ci_pai, variant, text} =>
            match_cipai(&rhyme_dict, cipai_file.as_str(), ci_pai, variant, text)?,
    }

    Ok(())
}
