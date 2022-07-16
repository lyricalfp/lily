use std::iter;

use crate::lines::Lines;

use super::cursor::{LayoutK, Token, TokenK};

pub fn split(
    i: impl Iterator<Item = Token> + Clone,
) -> (
    impl Iterator<Item = Token>,
    impl Iterator<Item = Token>,
    impl Iterator<Item = Token>,
) {
    let j = i.clone();
    let k = i.clone();
    (
        i.filter(|token| !matches!(token.kind, TokenK::Comment(_) | TokenK::Whitespace)),
        j.filter(|token| !matches!(token.kind, TokenK::Comment(_) | TokenK::Whitespace)),
        k.filter(|token| matches!(token.kind, TokenK::Comment(_) | TokenK::Whitespace)),
    )
}

pub fn join(
    l: Lines,
    i: impl Iterator<Item = Token>,
    j: impl Iterator<Item = Token>,
    k: impl Iterator<Item = Token>,
) -> impl Iterator<Item = Token> {
    let mut i = i.peekable();
    let mut j = j.peekable();
    let mut k = k.peekable();
    let mut l = iter::once(Token {
        begin: l.eof_offset(),
        end: l.eof_offset(),
        kind: TokenK::Eof,
    });
    iter::from_fn(move || match (i.peek(), j.peek(), k.peek()) {
        (Some(x), Some(y), Some(z)) => match y.kind {
            TokenK::Layout(LayoutK::Begin) => {
                if x.end <= y.end && x.end <= z.end {
                    i.next()
                } else if y.end <= x.end && y.end <= z.end {
                    let kind = y.kind;
                    j.next();
                    Some(Token {
                        begin: z.begin,
                        end: z.begin,
                        kind,
                    })
                } else {
                    k.next()
                }
            }
            TokenK::Layout(LayoutK::Separator(_) | LayoutK::End) => {
                if y.end <= x.end && y.end <= z.end {
                    let kind = y.kind;
                    j.next();
                    Some(Token {
                        begin: z.begin,
                        end: z.begin,
                        kind,
                    })
                } else if x.end <= y.end && x.end <= z.end {
                    i.next()
                } else {
                    k.next()
                }
            }
            _ => unreachable!(),
        },

        (Some(x), None, Some(z)) => {
            if x.begin < z.begin {
                i.next()
            } else {
                k.next()
            }
        }
        (None, Some(y), Some(z)) => {
            let kind = y.kind;
            j.next();
            Some(Token {
                begin: z.begin,
                end: z.begin,
                kind,
            })
        }
        (Some(x), Some(y), None) => {
            if x.begin < y.begin {
                i.next()
            } else {
                j.next()
            }
        },

        (Some(_), None, None) => i.next(),
        (None, Some(_), None) => j.next(),
        (None, None, Some(_)) => k.next(),
        (None, None, None) => l.next(),
    })
}
