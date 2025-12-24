pub mod reader;
pub mod writer;

use std::error::Error;

type NormalResult = Result<(), Box<dyn Error>>;
