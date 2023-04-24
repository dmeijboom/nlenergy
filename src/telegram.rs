use std::{error::Error, fmt::Display, iter::Peekable, time::Duration};

use anyhow::{anyhow, Result};
use chrono::Local;
use logos::{Lexer, Logos};
use reqwest::Client;
use rust_decimal::Decimal;
use sled::{Db, Tree};
use tokio::time;

use crate::energy::{Joule, Rate, State};

const T1_IMPORT: &str = "1-0:2.8.1";
const T1_EXPORT: &str = "1-0:1.8.1";
const T2_IMPORT: &str = "1-0:2.8.2'";
const T2_EXPORT: &str = "1-0:1.8.2";
const RATE_INDICATOR: &str = "0-0:96.14.0";

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    Eof,
    UnexpectedToken,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

impl Default for ParseError {
    fn default() -> Self {
        Self::UnexpectedToken
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f\r]+|!.*", error = ParseError)]
enum Token {
    #[regex(r"/.*", |lex| lex.slice().trim()[1..].to_string())]
    Comment(String),

    #[regex(r"[0-9]-[0-9]:[0-9\.]+", |lex| lex.slice().to_string())]
    Code(String),

    #[regex(r"\([^)]*\)", |lex| {
        let s = lex.slice();
        s[1..s.len() - 1].to_string()
    })]
    Value(String),
}

macro_rules! expect {
    ($lhr:expr, Token::$variant:ident) => {
        match $lhr {
            Ok(Token::$variant(value)) => value,
            Ok(token) => return Err(anyhow!("unexpected token: {token:?}")),
            Err(e) => return Err(e.into()),
        }
    };
}

#[inline]
fn next(lex: &mut Peekable<Lexer<Token>>) -> Result<Token, ParseError> {
    lex.next().ok_or(ParseError::Eof)?
}

fn parse_kwh(value: &str) -> Result<Joule> {
    if !value.ends_with("*kWh") {
        return Err(anyhow!("invalid value: {value}"));
    }

    let value = value[..value.len() - 4].parse::<Decimal>()?;
    Joule::from_kwh(value).ok_or_else(|| anyhow!("rounding error"))
}

async fn fetch_telegram(client: &Client, endpoint: &str) -> Result<String> {
    Ok(client.get(endpoint).send().await?.text().await?)
}

fn upsert(tree: &Tree, state: State) -> Result<Option<State>> {
    let checksum = state.checksum();

    if tree.contains_key(&checksum)? {
        return Ok(None);
    }

    tree.insert::<_, Vec<u8>>(checksum, (&state).into())?;

    Ok(Some(state))
}

async fn tick(tree: &Tree, client: &Client, endpoint: &str) -> Result<Option<State>> {
    let raw = fetch_telegram(client, endpoint).await?;
    let mut lex = Token::lexer(&raw).peekable();

    expect!(next(&mut lex), Token::Comment);

    let mut current_rate = None;
    let mut t1 = Joule(0);
    let mut t2 = Joule(0);

    while let Some(token) = lex.next() {
        let code = expect!(token, Token::Code);
        let value = &expect!(next(&mut lex), Token::Value);

        match code.as_str() {
            T1_EXPORT => {
                t1 += parse_kwh(value)?;
            }
            T1_IMPORT => {
                t1 -= parse_kwh(value)?;
            }
            T2_EXPORT => {
                t2 += parse_kwh(value)?;
            }
            T2_IMPORT => {
                t2 += parse_kwh(value)?;
            }
            RATE_INDICATOR => {
                current_rate = Some(value.parse::<u8>()?);
            }
            // skip other values
            _ => {
                while let Some(Ok(Token::Value(_))) = lex.peek() {
                    next(&mut lex)?;
                }
            }
        }
    }

    let time = Local::now();
    let state = match current_rate {
        Some(1) => State {
            rate: Rate::Normal,
            energy: t1,
            time,
        },
        Some(2) => State {
            rate: Rate::Normal,
            energy: t2,
            time,
        },
        Some(_) => return Err(anyhow!("unknown rate type: {current_rate:?}")),
        None => return Err(anyhow!("no rate indicator found")),
    };

    upsert(tree, state)
}

pub async fn cmd(db: Db, endpoint: String) -> Result<()> {
    let client = Client::new();
    let mut interval = time::interval(Duration::from_secs(1));
    let tree = db.open_tree("energy/history")?;

    loop {
        match tick(&tree, &client, &endpoint).await {
            Ok(Some(state)) => println!("new state: {state:?}"),
            Ok(None) => {}
            Err(e) => eprintln!("tick failed: {e:?}"),
        }

        interval.tick().await;
    }
}
