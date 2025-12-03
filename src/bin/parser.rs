use anyhow::Result;
use rhyme_checker::parser::rhyme_parser::{parse_pingshui, parse_cilin};
use rhyme_checker::parser::cipai_parser;
use std::fs::File;
use std::io::Write;

const DATA_DIR: &str = "data";
const OUTPUT_DIR: &str = "data/bin";

fn main() -> Result<()> {
    println!("Parsing all data ...");

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(OUTPUT_DIR)?;

    parse_rhyme_dicts()?;
    parse_and_save_cipai()?;

    Ok(())
}

fn parse_rhyme_dicts() -> Result<()> {
    let dicts = ["pingshui", "cilin", "xinyun"];

    for dict_name in dicts {
        println!("Parsing {} rhyme dictionary...", dict_name);

        let input_file = match dict_name {
            "pingshui" => format!("{}/rhyme/Pingshui_Rhyme.json", DATA_DIR),
            "cilin" => format!("{}/rhyme/Cilin_Rhyme.json", DATA_DIR),
            "xinyun" => format!("{}/rhyme/Xinyun_Rhyme.json", DATA_DIR),
            _ => unreachable!(),
        };

        let rhyme_dict = match dict_name {
            "pingshui" => parse_pingshui(&input_file)?,
            "cilin" | "xinyun" => parse_cilin(&input_file)?,
            _ => unreachable!(),
        };

        // Save as binary (using bincode with serde)
        let bin_output = format!("{}/{}_rhyme.bin", OUTPUT_DIR, dict_name);
        let bin_data = bincode::serde::encode_to_vec(&rhyme_dict, bincode::config::standard())?;
        let mut file = File::create(&bin_output)?;
        file.write_all(&bin_data)?;
        println!("Saved bin file {}", bin_output);
    }

    Ok(())
}

fn parse_and_save_cipai() -> Result<()> {
    println!("Parsing cipai data...");

    let input_file = format!("{}/cipai/cipai.xml", DATA_DIR);
    let cipai_list = cipai_parser::parse_cipai(&input_file)?;

    println!("Found {} cipai entries", cipai_list.len());

    // Save as binary (using bincode with serde)
    let bin_output = format!("{}/cipai.bin", OUTPUT_DIR);
    let bin_data = bincode::serde::encode_to_vec(&cipai_list, bincode::config::standard())?;
    let mut file = File::create(&bin_output)?;
    file.write_all(&bin_data)?;
    println!("Saved bin file {}", bin_output);

    Ok(())
}
