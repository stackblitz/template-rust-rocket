use std::borrow::Cow;
use unicode_xid::UnicodeXID;

use ext::IntoOwned;
use uri::{Uri, Origin};

use self::Error::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Static,
    Single,
    Multi,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    Path,
    Query,
    Data,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RouteSegment<'a> {
    pub string: Cow<'a, str>,
    pub kind: Kind,
    pub source: Source,
    pub name: Cow<'a, str>,
    pub index: Option<usize>,
}

impl<'a> IntoOwned for RouteSegment<'a> {
    type Owned = RouteSegment<'static>;

    #[inline]
    fn into_owned(self) -> Self::Owned {
        RouteSegment {
            string: IntoOwned::into_owned(self.string),
            kind: self.kind,
            source: self.source,
            name: IntoOwned::into_owned(self.name),
            index: self.index,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error<'a> {
    Empty,
    Ident(&'a str),
    Ignored,
    MissingClose,
    Malformed,
    Uri,
    Trailing(&'a str)
}

pub type SResult<'a> = Result<RouteSegment<'a>, (&'a str, Error<'a>)>;

#[inline]
fn is_ident_start(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || (c > '\x7f' && UnicodeXID::is_xid_start(c))
}

#[inline]
fn is_ident_continue(c: char) -> bool {
    ('a' <= c && c <= 'z')
        || ('A' <= c && c <= 'Z')
        || c == '_'
        || ('0' <= c && c <= '9')
        || (c > '\x7f' && UnicodeXID::is_xid_continue(c))
}

fn is_valid_ident(string: &str) -> bool {
    let mut chars = string.chars();
    match chars.next() {
        Some(c) => is_ident_start(c) && chars.all(is_ident_continue),
        None => false
    }
}

impl<'a> RouteSegment<'a> {
    pub fn parse_one(segment: &str) -> Result<RouteSegment, Error> {
        let (string, source, index) = (segment.into(), Source::Unknown, None);

        // Check if this is a dynamic param. If so, check its well-formedness.
        if segment.starts_with('<') && segment.ends_with('>') {
            let mut kind = Kind::Single;
            let mut name = &segment[1..(segment.len() - 1)];
            if name.ends_with("..") {
                kind = Kind::Multi;
                name = &name[..(name.len() - 2)];
            }

            if name.is_empty() {
                return Err(Empty);
            } else if !is_valid_ident(name) {
                return Err(Ident(name));
            } else if name == "_" {
                return Err(Ignored);
            }

            let name = name.into();
            return Ok(RouteSegment { string, source, name, kind, index });
        } else if segment.is_empty() {
            return Err(Empty);
        } else if segment.starts_with('<') && segment.len() > 1
                && !segment[1..].contains('<') && !segment[1..].contains('>') {
            return Err(MissingClose);
        } else if segment.contains('>') || segment.contains('<') {
            return Err(Malformed);
        } else if Uri::percent_encode(segment) != segment
                || Uri::percent_decode_lossy(segment.as_bytes()) != segment
                || segment.contains('+') {
            return Err(Uri);
        }

        Ok(RouteSegment {
            string, source, index,
            name: segment.into(),
            kind: Kind::Static,
        })
    }

    pub fn parse_many(
        string: &str,
        source: Source,
    ) -> impl Iterator<Item = SResult> {
        let sep = match source {
            Source::Query => '&',
            _ => '/',
        };

        let mut last_multi_seg: Option<&str> = None;
        string.split(sep).filter(|s| !s.is_empty()).enumerate().map(move |(i, seg)| {
            if let Some(multi_seg) = last_multi_seg {
                return Err((seg, Trailing(multi_seg)));
            }

            let mut parsed = Self::parse_one(seg).map_err(|e| (seg, e))?;
            if parsed.kind == Kind::Multi {
                last_multi_seg = Some(seg);
            }

            parsed.index = Some(i);
            parsed.source = source;
            Ok(parsed)
        })
    }

    pub fn parse_path(uri: &'a Origin) -> impl Iterator<Item = SResult<'a>> {
        Self::parse_many(uri.path(), Source::Path)
    }

    pub fn parse_query(uri: &'a Origin) -> Option<impl Iterator<Item = SResult<'a>>> {
        uri.query().map(|q| Self::parse_many(q, Source::Query))
    }
}
