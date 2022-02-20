use lazy_static::lazy_static;
use phf::phf_set;
use regex::Regex;
use substring::Substring;
use crate::{WCode, Token, WSection, FunctionParameter};
use crate::stdlib::FUNCTIONS;

pub fn lexer(code: &str) -> Vec<WSection> {
    lazy_static! {
        static ref RE: Regex = Regex::new(" <- ").unwrap();
        static ref SPECIALS: phf::Set<&'static str> = phf_set! {
            ")",
            "("
        };
    }

    fn inner(code: &str, containers: &Vec<String>) -> WCode {
        code.split(' ')
            .map(|x| match x.parse::<f64>() {
                Ok(n) => Token::Value(n),
                Err(_) => {
                    let cleared = x.chars().filter(|&x| x != '\n').collect::<String>();
                    let mut chars = cleared.chars();

                    if containers.iter().any(|name| *name == cleared) {
                        Token::Container(cleared)
                    } else if cleared.len() > 1 && chars.nth(0).unwrap() == '#' {
                        if let Ok(index) = cleared[1..].parse::<usize>() {
                            Token::Parameter(FunctionParameter::Exact(index))
                        } else if chars.nth(0).unwrap() == 'n' && cleared.len() == 2 {
                            Token::Parameter(FunctionParameter::Remaining)
                        } else {
                            Token::Atom(cleared)
                        }
                    } else if cleared.len() > 2
                        && chars.nth(0).unwrap() == '`'
                        && chars.last().unwrap() == '`'
                    {
                        let function = cleared.substring(1, cleared.len() - 1);

                        Token::FunctionLiteral(
                            *FUNCTIONS
                                .get(function)
                                .unwrap_or_else(|| panic!("Unknown function: {:?}", function)),
                        )
                    } else if SPECIALS.contains(&cleared) {
                        Token::Special(cleared)
                    } else {
                        match FUNCTIONS.get(&cleared) {
                            Some(x) => Token::Function(*x),
                            None => Token::Atom(x.to_string()),
                        }
                    }
                }
            })
            .collect()
    }

    let mut containers = vec![];

    code.split('\n')
        .filter(|&x| x.trim() != "" && x != "\n")
        .map(|line| match RE.find(line) {
            Some(pos) => {

                let container = line[..pos.start()].to_string();
                let code = line[pos.end()..].to_string();

                containers.push(container.clone());
                WSection {
                    container: Some(container),
                    code: inner(&code, &containers),
                }
            }
            None => WSection {
                container: None,
                code: inner(line, &containers),
            },
        })
        .collect()
}
