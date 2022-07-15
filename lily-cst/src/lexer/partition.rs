use std::{cmp::Ordering, iter};

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
        (Some(x), Some(y), Some(z)) => {
            let initial = if let TokenK::Layout(LayoutK::Separator | LayoutK::End) = y.kind {
                std::cmp::min_by_key((1, y), (0, x), |(_, token)| token.end)
            } else {
                std::cmp::min_by_key((0, x), (1, y), |(_, token)| token.end)
            };
            let (index, _) = std::cmp::min_by_key(initial, (2, z), |(_, token)| token.end);
            if index == 0 {
                i.next()
            } else if index == 1 {
                let kind = y.kind;
                j.next();
                Some(Token {
                    begin: z.begin,
                    end: z.begin,
                    kind,
                })
            } else if index == 2 {
                k.next()
            } else {
                unreachable!()
            }
        }

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
        // TODO: determine if this is truly the case
        (Some(_), Some(_), None) => unreachable!(),

        (Some(_), None, None) => i.next(),
        (None, Some(_), None) => j.next(),
        (None, None, Some(_)) => k.next(),
        (None, None, None) => l.next(),
    })
}
