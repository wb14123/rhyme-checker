use std::collections::HashMap;
use crate::core::tone::BasicTone;

pub type RhymeId = i8;

pub struct Rhyme {
    pub id: RhymeId,
    pub name: String,
    pub tone: BasicTone,
}

impl PartialEq for Rhyme {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct RhymeDict<'a> {
    chars_to_rhymes: HashMap<char, &'a [&'a Rhyme]>,
    rhyme_to_chars: HashMap<RhymeId, &'a [char]>,
    rhyme_map: HashMap<RhymeId, &'a Rhyme>
}

impl<'a> RhymeDict<'a> {

    pub fn get_chars_by_rhyme(&self, id: &RhymeId) -> &[char] {
        self.rhyme_to_chars.get(&id).copied().unwrap_or(&[])
    }

    pub fn get_rhyme_by_id(&self, id: &RhymeId) -> Option<&Rhyme> {
        self.rhyme_map.get(&id).copied()
    }

    pub fn get_rhymes_by_char(&self, c: &char) -> &[&Rhyme] {
        self.chars_to_rhymes.get(&c).copied().unwrap_or(&[])
    }
}