//! Convenience enum for converting to/from strings representing the latest, earliest, and pending
//! blocks

use failure::Fail;
use regex::{Regex, RegexSet};

pub enum BlockString {
    Earliest,
    Latest,
    Pending
}

#[derive(Fail, Debug)]
pub enum BlockStringError {
    #[fail(display = "Error creating regex for blockstring match: {}", _0)]
    RegexCreate(#[cause] regex::Error),
    #[fail(display = "Error converting to blockstring; must be one of 'latest', 'earliest', or 'pending'")]
    Convert,
}

impl From<regex::Error> for BlockStringError {
    fn from(err: regex::Error) -> BlockStringError {
        BlockStringError::RegexCreate(err)
    }
}

impl From<&BlockString> for String {
    fn from(blk: &BlockString) -> String {
        match blk {
            BlockString::Earliest => "earliest".to_string(),
            BlockString::Latest => "latest".to_string(),
            BlockString::Pending => "pending".to_string()
        }
    }
}

impl From<BlockString> for String {
    fn from(blk: BlockString) -> String {
        match blk {
            BlockString::Earliest => "earliest".to_string(),
            BlockString::Latest => "latest".to_string(),
            BlockString::Pending => "pending".to_string()
        }
    }
}

impl BlockString {
    pub fn new(string: String) -> Result<BlockString, BlockStringError> {
        
        
        let re_set = RegexSet::new(&[
            // r"^(?:earliest|pending|latest)$",
            r"^earliest$",
            r"^latest$",
            r"^pending$"
        ]);
        
        let re_set: regex::RegexSet = re_set.map_err(|e: regex::Error | BlockStringError::from(e))?;
            
            //map_err(|e| e.into())?;
        
        let get_str = |matches: regex::SetMatches| {
            if matches.matched(0) { BlockString::Earliest }
            else if matches.matched(1) { BlockString::Latest }
            else if matches.matched(2) { BlockString::Pending }
            else { unreachable!() }
        };
            
        
        let matches = re_set.matches(&string.to_lowercase());

        if !matches.matched_any() {
            Err(BlockStringError::Convert)
        } else {
            Ok(get_str(matches))
        }
    } 

    pub fn to_str(&self) -> String {
        String::from(self)
    }
}
