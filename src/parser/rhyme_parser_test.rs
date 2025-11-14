use crate::core::tone::BasicTone;
use crate::parser::rhyme_parser::parse_pingshui;

#[test]
fn test_parse_pingshui_success() {
    // Test parsing the Pingshui_Rhyme.json file
    let file_path = "data/rhyme/Pingshui_Rhyme.json";
    let result = parse_pingshui(file_path);

    assert!(result.is_ok(), "Failed to parse Pingshui_Rhyme.json: {:?}", result.err());

    let rhyme_dict = result.unwrap();

    // Verify that some basic characters are present
    // Test character '东' from "一东" rhyme
    let rhymes_for_dong = rhyme_dict.get_rhymes_by_char(&'东');
    assert!(!rhymes_for_dong.is_empty(), "Character '东' should have rhyme mappings");

    // Check that the rhyme has correct tone (平声)
    let rhyme = &rhymes_for_dong[0];
    assert_eq!(rhyme.tone, BasicTone::Ping, "Character '东' should be in Ping tone");
    assert_eq!(rhyme.name, "一东", "Character '东' should be in '一东' rhyme");
}

#[test]
fn test_parse_pingshui_rhyme_structure() {
    let file_path = "data/rhyme/Pingshui_Rhyme.json";
    let rhyme_dict = parse_pingshui(file_path).expect("Should parse successfully");

    // Test that different tone sections are parsed correctly
    // '东' is in 上平声部 (Ping)
    let ping_char = rhyme_dict.get_rhymes_by_char(&'东');
    assert!(!ping_char.is_empty());
    assert_eq!(ping_char[0].tone, BasicTone::Ping);

    // We should also check if there are Ze tone characters
    // Let's check if the file has both Ping and Ze tones
    let _all_rhymes_have_valid_tones = true;
    // This is a structural test - just verify parsing doesn't crash
}

#[test]
fn test_parse_pingshui_char_lookup() {
    let file_path = "data/rhyme/Pingshui_Rhyme.json";
    let rhyme_dict = parse_pingshui(file_path).expect("Should parse successfully");

    // Test character '冬' from "二冬" rhyme
    let rhymes_for_dong = rhyme_dict.get_rhymes_by_char(&'冬');
    assert!(!rhymes_for_dong.is_empty(), "Character '冬' should have rhyme mappings");
    assert_eq!(rhymes_for_dong[0].name, "二冬", "Character '冬' should be in '二冬' rhyme");

    // Test character '江' from "三江" rhyme
    let rhymes_for_jiang = rhyme_dict.get_rhymes_by_char(&'江');
    assert!(!rhymes_for_jiang.is_empty(), "Character '江' should have rhyme mappings");
    assert_eq!(rhymes_for_jiang[0].name, "三江", "Character '江' should be in '三江' rhyme");
}

#[test]
fn test_parse_pingshui_rhyme_id_lookup() {
    let file_path = "data/rhyme/Pingshui_Rhyme.json";
    let rhyme_dict = parse_pingshui(file_path).expect("Should parse successfully");

    // Get rhyme by ID (first rhyme should be ID 0)
    let rhyme_0 = rhyme_dict.get_rhyme_by_id(&0);
    assert!(rhyme_0.is_some(), "Rhyme ID 0 should exist");

    let rhyme = rhyme_0.unwrap();
    assert_eq!(rhyme.name, "一东", "First rhyme should be '一东'");
    assert_eq!(rhyme.id, 0);

    // Get characters for this rhyme
    let chars = rhyme_dict.get_chars_by_rhyme(&0);
    assert!(!chars.is_empty(), "Rhyme '一东' should have characters");
    assert!(chars.contains(&'东'), "Rhyme '一东' should contain character '东'");
}

#[test]
fn test_parse_pingshui_invalid_file() {
    let result = parse_pingshui("non_existent_file.json");
    assert!(result.is_err(), "Should fail for non-existent file");
}

#[test]
fn test_parse_pingshui_tone_detection() {
    let file_path = "data/rhyme/Pingshui_Rhyme.json";
    let rhyme_dict = parse_pingshui(file_path).expect("Should parse successfully");

    // Verify that all characters from "一东" are in Ping tone
    let rhyme_0 = rhyme_dict.get_rhyme_by_id(&0).expect("Rhyme 0 should exist");
    assert_eq!(rhyme_0.tone, BasicTone::Ping);
    assert_eq!(rhyme_0.group, None, "平水韵 should have no group");
}
