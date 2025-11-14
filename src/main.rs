mod core;
mod parser;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use parser::rhyme_parser::parse_pingshui;
use parser::cipai_parser::parse_cipai;
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
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查询汉字的韵部信息
    QueryCharRhyme {
        /// 韵书文件夹路径
        #[arg(short, long, default_value = "data/rhyme")]
        dict_dir: String,

        /// 要查询的汉字
        #[arg(value_name = "CHAR")]
        character: String,

        /// 显示该韵部的所有汉字
        #[arg(short, long)]
        show_all: bool,

        /// 韵书类型
        #[arg(short = 't', long, value_enum, default_value = "pingshui")]
        dict_type: DictType,
    },

    /// 查询词牌信息
    QueryCiPai {
        /// 词牌文件路径
        #[arg(short, long, default_value = "data/cipai/cipai.xml")]
        file: String,

        /// 要查询的词牌名
        #[arg(value_name = "NAME")]
        name: String,
    },
}

fn query_char_rhyme(dict_dir: &str, character: &str, show_all: bool, dict_type: &DictType) -> Result<()> {
    let rhyme_dict = match dict_type {
        DictType::Pingshui => parse_pingshui(format!("{}/Pingshui_Rhyme.json", dict_dir).as_str())?,
        DictType::Cilin => parse_cilin(format!("{}/Cilin_Rhyme.json", dict_dir).as_str())?,
    };

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

fn query_cipai(file: &str, name: &str) -> Result<()> {
    let cipai_list = parse_cipai(file)?;

    let matching_cipai: Vec<_> = cipai_list
        .iter()
        .filter(|cipai| cipai.names.iter().any(|n| n.contains(name)))
        .collect();

    if matching_cipai.is_empty() {
        println!("未找到词牌: {}", name);
        return Ok(());
    }

    for (i, cipai) in matching_cipai.iter().enumerate() {
        if i > 0 {
            println!("\n{}", "=".repeat(60));
        }
        print!("{}", cipai);
    }
    println!();

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::QueryCharRhyme {
            dict_dir,
            character,
            show_all,
            dict_type,
        } => query_char_rhyme(dict_dir, character, *show_all, dict_type)?,
        Commands::QueryCiPai { file, name } => query_cipai(file, name)?,
    }

    Ok(())
}
