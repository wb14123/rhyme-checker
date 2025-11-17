use crate::core::tone::BasicTone;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub type RhymeId = i8;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rhyme {
    pub id: RhymeId,
    pub name: String,
    pub group: Option<String>, //韵部，如果为空则不检查韵部
    pub tone: BasicTone,
}

impl fmt::Display for Rhyme {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.tone)?;
        if self.group.is_some() {
            write!(f, ", {}", self.group.as_ref().unwrap())?;
        }
        Ok(())
    }
}

impl PartialEq for Rhyme {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Rhyme {}

impl Hash for Rhyme {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// TODO: #[derive(Serialize, Deserialize)]
pub struct RhymeDict {
    chars_to_rhymes: HashMap<char, Vec<Arc<Rhyme>>>,
    rhyme_to_chars: HashMap<RhymeId, Vec<char>>,
}

impl RhymeDict {

    pub fn new(rhyme_chars: Vec<Vec<char>>, rhymes: Vec<Arc<Rhyme>>) -> Result<RhymeDict> {

        let mut chars_to_rhymes = HashMap::new();

        for rid in 0..rhyme_chars.len() {
            let chars = &rhyme_chars[rid];
            for char in chars {
                if chars_to_rhymes.get(char).is_none() {
                    chars_to_rhymes.insert(*char, vec![]);
                }
                chars_to_rhymes.get_mut(char).unwrap().push(
                    rhymes.get(rid).context("Rhyme for char not found in rhyme map")?.clone());
            }
        }

        let rhyme_to_chars = rhyme_chars
            .into_iter().enumerate().map(|(k, v)| (k as RhymeId, v)).collect();

        Ok(RhymeDict { chars_to_rhymes, rhyme_to_chars})
    }

    pub fn get_chars_by_rhyme(&self, id: &RhymeId) -> &[char] {
        self.rhyme_to_chars
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_rhymes_by_char(&self, c: &char) -> &[Arc<Rhyme>] {
        self.chars_to_rhymes
            .get(&c)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}