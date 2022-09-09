use std::fmt::{Debug, Display, Formatter, Write};
use std::num::ParseIntError;
use std::str::FromStr;
use std::error::Error;
use raqote::Color;

#[derive(Debug)]
pub struct CliColor {
    color: (u8, u8, u8, u8)
}

impl CliColor {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        CliColor {
            color: (a, r, g, b)
        }
    }
}

impl Into<Color> for CliColor {
    fn into(self) -> Color {
        let (a, r, g, b) = self.color;
        Color::new(a, r, g, b)
    }
}

impl Display for CliColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (a, r, g, b) = self.color;
        f.write_str(&format!("{},{},{},{}", a, r, g, b))
    }
}

#[derive(Debug)]
pub enum CliColorErr {
    ParseValueError(String),
    NotEnoughValuesError(usize)
}

impl Display for CliColorErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CliColorErr::ParseValueError(s) => f.write_str(&format!("Failed to parse input color '{}'", s)),
            CliColorErr::NotEnoughValuesError(v) => f.write_str(&format!("Not enough values provided for color, 4 needed {} provided", v))
        }
    }
}

impl FromStr for CliColor {
    type Err = CliColorErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let argb = s.split(",").map(|c|c.parse::<u8>());
        let mut values = Vec::new();
        for v in argb {
            match v {
                Ok(c) => values.push(c),
                Err(_e) => return Err(Self::Err::ParseValueError(s.to_string()))
            }
        };
        if values.len() == 4 {
            Ok(
                CliColor::new(
                    values[0],
                    values[1],
                    values[2],
                    values[3]
                )
            )
        }else{
            Err(Self::Err::NotEnoughValuesError(values.len()))
        }
    }
}

impl Error for CliColorErr {

}