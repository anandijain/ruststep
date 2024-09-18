use itertools::*;
use std::{cmp, fmt};

/// Identifier in EXPRESS language must be one of scopes described in
/// "Table 9 – Scope and identifier defining items"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeType {
    Entity,
    Alias,
    Function,
    Procedure,
    Query,
    Repeat,
    Rule,
    Schema,
    SubType,
    Type,
}

/// Scope declaration
///
/// Partial Order
/// --------------
/// Scope is partially ordered in terms of the sub-scope relation:
///
/// ```
/// # use espr::ir::*;
/// let root = Scope::root();
/// let schema = root.pushed(ScopeType::Schema, "schema");
/// assert!(root > schema); // schema scope is sub-scope of root scope
/// ```
///
/// Be sure that this is not total order:
///
/// ```
/// # use espr::ir::*;
/// let root = Scope::root();
/// let schema1 = root.pushed(ScopeType::Schema, "schema1");
/// let schema2 = root.pushed(ScopeType::Schema, "schema2");
///
/// // schema1 and schema2 are both sub-scope of root,
/// assert!(root > schema1);
/// assert!(root > schema2);
///
/// // but they are independent. Comparison always returns false:
/// assert!(!(schema1 <= schema2));
/// assert!(!(schema1 >= schema2));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Scope(Vec<(ScopeType, String)>);

// Custom debug output like: `schema.entity`
impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(|(_ty, name)| name).join("."))
    }
}

// Custom debug output like: `Scope(schema[Schema].entity[Entity])`
impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope(")?;
        for (i, (ty, name)) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            write!(f, "{}[{:?}]", name, ty)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        for (lhs, rhs) in self.0.iter().zip(other.0.iter()) {
            if lhs != rhs {
                return None;
            }
        }
        Some(other.0.len().cmp(&self.0.len()))
    }
}

macro_rules! add_scope {
    ($f:ident, $ty:ident) => {
        #[doc = stringify!(Add $ty scope)]
        pub fn $f(&self, name: &str) -> Self {
            self.pushed(ScopeType::$ty, name)
        }
    };
}

impl Scope {
    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub fn pushed(&self, ty: ScopeType, name: &str) -> Self {
        let mut new = self.clone();
        new.0.push((ty, name.to_string()));
        new
    }

    add_scope!(entity, Entity);
    add_scope!(alias, Alias);
    add_scope!(function, Function);
    add_scope!(procedure, Procedure);
    add_scope!(query, Query);
    add_scope!(repeat, Repeat);
    add_scope!(rule, Rule);
    add_scope!(schema, Schema);
    add_scope!(subtype, SubType);
    add_scope!(r#type, Type);

    /// Pop the last scope
    ///
    /// Returns `None` when `self` is root.
    pub fn popped(&self) -> Option<Self> {
        let mut new = self.clone();
        let _current = new.0.pop()?;
        Some(new)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pub scope: Scope,
    pub ty: ScopeType,
    pub name: String,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.scope, self.name)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{}[{:?}]", self.scope, self.name, self.ty)
    }
}

macro_rules! new_path {
    ($f:ident, $ty:ident) => {
        #[doc = stringify!(Add $ty scope)]
        pub fn $f(scope: &Scope, name: &str) -> Self {
            Path::new(scope, ScopeType::$ty, name)
        }
    };
}

impl Path {
    pub fn new(scope: &Scope, ty: ScopeType, name: &str) -> Self {
        Path {
            scope: scope.clone(),
            ty,
            name: name.to_string(),
        }
    }

    new_path!(entity, Entity);
    new_path!(alias, Alias);
    new_path!(function, Function);
    new_path!(procedure, Procedure);
    new_path!(query, Query);
    new_path!(repeat, Repeat);
    new_path!(rule, Rule);
    new_path!(schema, Schema);
    new_path!(subtype, SubType);
    new_path!(r#type, Type);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope() {
        let root = Scope::root();
        assert_eq!(format!("{}", root), "");

        let schema = root.pushed(ScopeType::Schema, "schema1");
        assert_eq!(format!("{}", schema), "schema1");
        assert_eq!(format!("{:?}", schema), "Scope(schema1[Schema])");

        let entity = schema.pushed(ScopeType::Entity, "entity1");
        assert_eq!(format!("{}", entity), "schema1.entity1");
        assert_eq!(
            format!("{:?}", entity),
            "Scope(schema1[Schema].entity1[Entity])"
        );
    }
}
