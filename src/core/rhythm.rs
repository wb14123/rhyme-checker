use std::collections::HashMap;
use crate::core::tone::BasicTone;

pub type RhythmId = i8;

pub struct Rhythm {
    pub id: RhythmId,
    pub name: String,
    pub tone: BasicTone,
}

impl PartialEq for Rhythm {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct RhythmDict<'a> {
    chars_to_rhythms: HashMap<char, &'a [&'a Rhythm]>,
    rhythm_to_chars: HashMap<RhythmId, &'a [char]>,
    rhythm_map: HashMap<RhythmId, &'a Rhythm>
}

impl<'a> RhythmDict<'a> {

    pub fn get_chars_by_rhythm(&self, id: &RhythmId) -> &[char] {
        self.rhythm_to_chars.get(&id).copied().unwrap_or(&[])
    }

    pub fn get_rhythm_by_id(&self, id: &RhythmId) -> Option<&Rhythm> {
        self.rhythm_map.get(&id).copied()
    }

    pub fn get_rhythms_by_char(&self, c: &char) -> &[&Rhythm] {
        self.chars_to_rhythms.get(&c).copied().unwrap_or(&[])
    }
}