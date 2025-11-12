mod core;
mod parser;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use parser::rhyme_parser::parse_pingshui;

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
        /// 韵书文件路径
        #[arg(short, long, default_value = "data/rhyme/Pingshui_Rhyme.json")]
        dict: String,

        /// 要查询的汉字
        #[arg(value_name = "CHAR")]
        character: String,

        /// 显示该韵部的所有汉字
        #[arg(short, long)]
        show_all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::QueryCharRhyme {
            dict,
            character,
            show_all,
        } => {
            let rhyme_dict = parse_pingshui(dict)?;

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

                if *show_all {
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
        }
    }

    Ok(())
}
