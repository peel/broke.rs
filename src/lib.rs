use clap::ValueEnum;

pub mod data;

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Mode {
    Pub,
    Sub,
}
