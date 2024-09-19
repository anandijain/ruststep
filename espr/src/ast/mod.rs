//! Abstract Syntax Tree (AST) of EXPRESS Language

mod algorithm;
mod entity;
mod error;
mod expression;
mod schema;
mod types;

pub use algorithm::*;
pub use entity::*;
pub use error::*;
pub use expression::*;
pub use schema::*;
pub use types::*;

use crate::parser::{combinator::*, *};
use nom::Finish;

pub trait Component: Sized {
    fn parse(input: &str) -> Result<(Self, Vec<Remark>), TokenizeFailed>;
    fn from_str(input: &str) -> Result<Self, TokenizeFailed> {
        let (component, _remarks) = Self::parse(input)?;
        Ok(component)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! derive_ast_component {
    ($component:ty, $parser:path) => {
        impl $crate::ast::Component for $component {
            fn parse(
                input: &str,
            ) -> Result<(Self, Vec<$crate::ast::Remark>), $crate::ast::TokenizeFailed> {
                use nom::Finish;
                let input = input.trim();
                let (_input, parsed) = $parser(input)
                    .finish()
                    .map_err(|err| $crate::ast::TokenizeFailed::new(input, err))?;
                Ok(parsed)
            }
        }
    };
}

/// Remarks in EXPRESS input, `(* ... *)` or `-- ...`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remark {
    pub tag: Option<Vec<String>>,
    pub remark: String,
}

/// Entire syntax tree parsed from EXPRESS Language string
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTree {
    pub schemas: Vec<Schema>,
    pub remarks: Vec<Remark>,
}

impl SyntaxTree {
    pub fn parse(input: &str) -> Result<Self, nom::error::VerboseError<&str>> {
        let (residual, (schemas, remarks)) = tuple((spaces, many1(schema_decl), spaces))
            .map(|(_start_space, schemas, _end_space)| schemas)
            .parse(input)
            .finish()?;
        assert!(residual.is_empty());
        Ok(SyntaxTree { schemas, remarks })
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

#[cfg(test)]
mod tests {

    #[test]
    fn parse_remarks() {
        let st = super::SyntaxTree::parse(
            r#"
            SCHEMA one;
              ENTITY first;
                m_ref : second;
                fattr : STRING;
              END_ENTITY; -- first
              ENTITY second;
                sattr : STRING;
              END_ENTITY; -- second
              (* this is the example! *)
            END_SCHEMA;

            (* Hey! *)

            SCHEMA geometry0;
              ENTITY point;
                x, (* y, *) z: REAL; -- skip y
              END_ENTITY;
            END_SCHEMA;
            "#,
        )
        .unwrap();
        dbg!(&st);
        assert_eq!(st.remarks.len(), 6);
    }
}
