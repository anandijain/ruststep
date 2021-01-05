//! Tokenize EXPRESS language into [SyntaxTree]
//!
//! This submodule responsible for tokenize of EXPRESS language input into a [SyntaxTree] struct.
//! Following steps of compile, i.e. semantics analysis and Rust code generation will be handled by
//! other submodules.
//!
//! This submodule is based on [nom](https://github.com/Geal/nom) parser combinater.
//!
//! Example
//! --------
//!
//! EXPRESS Language string is parsed into [SyntaxTree]:
//!
//! ```
//! let schemas = espr::parser::SyntaxTree::parse(r#"
//! SCHEMA one;
//!   ENTITY first;
//!     m_ref : second;
//!     fattr : STRING;
//!   END_ENTITY;
//!   ENTITY second;
//!     sattr : STRING;
//!   END_ENTITY;
//! END_SCHEMA;
//!
//! SCHEMA geometry0;
//!   ENTITY point;
//!     x, y, z: REAL;
//!   END_ENTITY;
//! END_SCHEMA;
//! "#.trim()).unwrap();
//! ```

pub mod entity;
pub mod expression;
pub mod literal;
pub mod remark;
pub mod schema;
pub mod simple_data_type;

use nom::{
    branch::*, bytes::complete::*, character::complete::*, multi::*, sequence::*, Finish, IResult,
    Parser,
};
use schema::*;

/// Entire syntax tree parsed from EXPRESS Language string
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTree {
    pub schemas: Vec<Schema>,
}

impl SyntaxTree {
    pub fn parse(input: &str) -> Result<Self, nom::error::Error<&str>> {
        let (_residual, schemas) = tuple((
            multispace0,
            separated_list1(multispace1, schema),
            multispace0,
        ))
        .map(|(_, schemas, _)| schemas)
        .parse(input)
        .finish()?;
        // FIXME should check residual here
        Ok(Self { schemas })
    }

    // Example syntax tree for easy testing
    //
    // FIXME Replace by e.g. proptest
    // https://github.com/AltSysrq/proptest
    #[allow(dead_code)]
    pub(crate) fn example() -> Self {
        Self::parse(
            r#"
            SCHEMA one;
              ENTITY first;
                m_ref : second;
                fattr : STRING;
              END_ENTITY;
              ENTITY second;
                sattr : STRING;
              END_ENTITY;
            END_SCHEMA;

            SCHEMA geometry0;
              ENTITY point;
                x, y, z: REAL;
              END_ENTITY;
            END_SCHEMA;
            "#
            .trim(),
        )
        .unwrap()
    }
}

/// 128 letter = `a` | `b` | `c` | `d` | `e` | `f` | `g` | `h` | `i` | `j` | `k` | `l` |`m` | `n` | `o` | `p` | `q` | `r` | `s` | `t` | `u` | `v` | `w` | `x` |`y` | `z` .
pub fn letter(input: &str) -> IResult<&str, char> {
    satisfy(|c| matches!(c, 'a'..='z')).parse(input)
}

/// 124 digit = `0` | `1` | `2` | `3` | `4` | `5` | `6` | `7` | `8` | `9` .
pub fn digit(input: &str) -> IResult<&str, char> {
    satisfy(|c| matches!(c, '0'..='9')).parse(input)
}

/// 143 simple_id = letter { letter | digit | `_` } .
pub fn simple_id(input: &str) -> IResult<&str, String> {
    tuple((letter, many0(alt((letter, digit, char('_'))))))
        .map(|(head, tail)| format!("{}{}", head, tail.into_iter().collect::<String>()))
        .parse(input)
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    #[test]
    fn letter() {
        let (residual, l) = super::letter("h").finish().unwrap();
        assert_eq!(l, 'h');
        assert_eq!(residual, "");

        let (residual, l) = super::letter("abc").finish().unwrap();
        assert_eq!(l, 'a');
        assert_eq!(residual, "bc");

        // Capital is not allowed
        assert!(super::letter("H").finish().is_err());
        // Number is not allowed
        assert!(super::letter("2").finish().is_err());
    }

    #[test]
    fn digit() {
        let (residual, l) = super::digit("123").finish().unwrap();
        assert_eq!(l, '1');
        assert_eq!(residual, "23");

        // Alphabets are not allowed
        assert!(super::digit("h").finish().is_err());
    }

    #[test]
    fn simple_id_valid() {
        let (residual, id) = super::simple_id("h").finish().unwrap();
        assert_eq!(id, "h");
        assert_eq!(residual, "");

        let (residual, id) = super::simple_id("homhom").finish().unwrap();
        assert_eq!(id, "homhom");
        assert_eq!(residual, "");

        let (residual, id) = super::simple_id("ho_mhom").finish().unwrap();
        assert_eq!(id, "ho_mhom");
        assert_eq!(residual, "");

        let (residual, id) = super::simple_id("h10o_1mh2om").finish().unwrap();
        assert_eq!(id, "h10o_1mh2om");
        assert_eq!(residual, "");
    }

    #[test]
    fn simple_id_invalid() {
        // Capital is not allowed
        assert!(super::simple_id("HomHom").finish().is_err());
        // `_` cannot use as first
        assert!(super::simple_id("_homhom").finish().is_err());
        // digit cannot use as first
        assert!(super::simple_id("1homhom").finish().is_err());
        // Empty is invlaid
        assert!(super::simple_id("").finish().is_err());
    }
}
