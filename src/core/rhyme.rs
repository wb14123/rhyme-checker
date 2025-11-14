use crate::core::tone::BasicTone;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

pub type RhymeId = i8;

#[derive(Serialize, Deserialize)]
pub struct Rhyme {
    pub id: RhymeId,
    pub name: String,
    pub group: Option<String>, //韵部，如果为空则不检查韵部
    pub tone: BasicTone,
}

impl fmt::Display for Rhyme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.tone)
    }
}

impl PartialEq for Rhyme {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// TODO: #[derive(Serialize, Deserialize)]
pub struct RhymeDict {
    chars_to_rhymes: HashMap<char, Vec<Arc<Rhyme>>>,
    rhyme_to_chars: HashMap<RhymeId, Vec<char>>,
    rhyme_map: HashMap<RhymeId, Arc<Rhyme>>
}

impl RhymeDict {

    pub fn new(rhyme_to_chars: HashMap<RhymeId, Vec<char>>, rhyme_map: HashMap<RhymeId, Arc<Rhyme>>) -> Result<RhymeDict> {

        let mut chars_to_rhymes = HashMap::new();

        for (rid, chars) in &rhyme_to_chars {
            for char in chars {
                if chars_to_rhymes.get(char).is_none() {
                    chars_to_rhymes.insert(*char, vec![]);
                }
                chars_to_rhymes.get_mut(char).unwrap().push(
                    rhyme_map.get(rid).context("Rhyme for char not found in rhyme map")?.clone());
            }
        }

        Ok(RhymeDict { chars_to_rhymes, rhyme_to_chars, rhyme_map })
    }

    pub fn get_chars_by_rhyme(&self, id: &RhymeId) -> &[char] {
        self.rhyme_to_chars
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_rhyme_by_id(&self, id: &RhymeId) -> Option<Arc<Rhyme>> {
        self.rhyme_map.get(&id).map(Arc::clone)
    }

    pub fn get_rhymes_by_char(&self, c: &char) -> &[Arc<Rhyme>] {
        self.chars_to_rhymes
            .get(&c)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}