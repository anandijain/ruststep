//! Partial complex entities described in ISO-10303-11 Annex B
use super::*;

use itertools::Itertools;
use std::cmp::Ordering;

#[cfg_attr(doc, katexit::katexit)]
/// Partial complex entity data type, e.g. $A \And B \And C$ in ISO document
///
/// Each component, e.g. $A$, will be represented by an index.
/// $\And$ operation is implemented by [std::ops::BitAnd] trait.
/// This satisfies following equations:
///
/// - $A \And A = A$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// assert_eq!(a.clone() & a.clone(), a);
/// ```
///
/// - $A \And B = B \And A$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b = PartialComplexEntity::new(&[2]);
/// assert_eq!(a.clone() & b.clone(), b & a);
/// ```
///
/// - $A \And (B \And C) = (A \And B) \And C = A \And B \And C$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b = PartialComplexEntity::new(&[3]);
/// let c = PartialComplexEntity::new(&[2]);
/// assert_eq!(
///     (a.clone() & b.clone()) & c.clone(),
///      a.clone() & (b.clone() & c.clone())
/// );
/// assert_eq!(a & b & c, PartialComplexEntity::new(&[1, 2, 3]));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialComplexEntity {
    /// Sorted and non-duplicated indices
    pub indices: Vec<usize>,
}

impl PartialOrd for PartialComplexEntity {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for PartialComplexEntity {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match Ord::cmp(&self.indices.len(), &rhs.indices.len()) {
            Ordering::Equal => Ord::cmp(&self.indices, &rhs.indices),
            a @ Ordering::Less | a @ Ordering::Greater => a,
        }
    }
}

impl PartialComplexEntity {
    pub fn new(indices: &[usize]) -> Self {
        PartialComplexEntity {
            indices: indices.iter().cloned().dedup().collect(),
        }
    }

    /// Restore Path from namespace index
    pub fn as_path(&self, ns: &Namespace) -> Vec<Path> {
        self.indices
            .iter()
            .map(|index| {
                let (path, _ast) = &ns[*index];
                path.clone()
            })
            .collect()
    }
}

impl From<&[usize]> for PartialComplexEntity {
    fn from(indices: &[usize]) -> PartialComplexEntity {
        PartialComplexEntity::new(indices)
    }
}

impl std::ops::BitAnd for PartialComplexEntity {
    type Output = Self;
    fn bitand(mut self, mut rhs: Self) -> Self {
        self.indices.append(&mut rhs.indices);
        self.indices.sort_unstable();
        PartialComplexEntity {
            indices: self.indices.into_iter().dedup().collect(),
        }
    }
}

#[cfg_attr(doc, katexit::katexit)]
/// Instantiable subtypes described by a list of partial complex entity, e.g. $[A, B & C]$
///
/// This has several operation described in ISO-10303-11 annex B,
/// $+$, $-$, $\And$, and $/$.
///
/// - $A + [B_1, B_2] = [B_1, B_2] + A = [A, B_1, B_2]$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b1 = PartialComplexEntity::new(&[2]);
/// let b2 = PartialComplexEntity::new(&[3]);
///
/// let ce = Instantiables::new(&[b1.clone(), b2.clone()]);
///
/// assert_eq!(a.clone() + ce, Instantiables::new(&[
///   a.clone(),
///   b1.clone(),
///   b2.clone(),
/// ]));
/// ```
///
/// - $A \And [B_1, B_2] = [B_1, B_2] \And A = [A \And B_1, A \And B_2]$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b1 = PartialComplexEntity::new(&[2]);
/// let b2 = PartialComplexEntity::new(&[3]);
///
/// let ce = Instantiables::new(&[b1.clone(), b2.clone()]);
///
/// assert_eq!(a.clone() & ce, Instantiables::new(&[
///   a.clone() & b1.clone(),
///   a.clone() & b2.clone(),
/// ]));
/// ```
///
/// - $[A_1, A_2] + [B_1, B_2] = [A_1, A_2, B_1, B_2]$
///
/// ```
/// # use espr::ir::*;
/// let a1 = PartialComplexEntity::new(&[1]);
/// let a2 = PartialComplexEntity::new(&[2]);
/// let b1 = PartialComplexEntity::new(&[3]);
/// let b2 = PartialComplexEntity::new(&[4]);
///
/// let ce1 = Instantiables::new(&[a1.clone(), a2.clone()]);
/// let ce2 = Instantiables::new(&[b1.clone(), b2.clone()]);
///
/// assert_eq!(ce1 + ce2, Instantiables::new(&[
///   a1.clone(),
///   a2.clone(),
///   b1.clone(),
///   b2.clone(),
/// ]));
/// ```
///
/// - $[A_1, A_2] \And [B_1, B_2] = [A_1 \And B_1, A_1 \And B_2, A_2 \And B_1, A_2 \And B_2]$
///
/// ```
/// # use espr::ir::*;
/// let a1 = PartialComplexEntity::new(&[1]);
/// let a2 = PartialComplexEntity::new(&[2]);
/// let b1 = PartialComplexEntity::new(&[3]);
/// let b2 = PartialComplexEntity::new(&[4]);
///
/// let ce1 = Instantiables::new(&[a1.clone(), a2.clone()]);
/// let ce2 = Instantiables::new(&[b1.clone(), b2.clone()]);
///
/// assert_eq!(ce1 & ce2, Instantiables::new(&[
///   a1.clone() & b1.clone(),
///   a1.clone() & b2.clone(),
///   a2.clone() & b1.clone(),
///   a2.clone() & b2.clone(),
/// ]));
/// ```
///
/// - $[A, A \And B, A \And C, A \And B \And D, B \And C, D] / A = [A, A \And B, A \And C, A \And B \And D]$
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b = PartialComplexEntity::new(&[2]);
/// let c = PartialComplexEntity::new(&[3]);
/// let d = PartialComplexEntity::new(&[4]);
///
/// let ce = Instantiables::new(&[
///   a.clone(),
///   a.clone() & b.clone(),
///   a.clone() & c.clone(),
///   a.clone() & b.clone() & d.clone(),
///   b.clone() & c.clone(),
///   d.clone()
/// ]);
///
/// assert_eq!(ce / a.clone(), Instantiables::new(&[
///   a.clone(),
///   a.clone() & b.clone(),
///   a.clone() & c.clone(),
///   a.clone() & b.clone() & d.clone(),
/// ]));
/// ```
///
/// - $ [A, A \And B, A \And C, A \And B \And D, B \And C, D]/[B, D] = [A \And B, A \And B \And D, B \And C, D] $
///
/// ```
/// # use espr::ir::*;
/// let a = PartialComplexEntity::new(&[1]);
/// let b = PartialComplexEntity::new(&[2]);
/// let c = PartialComplexEntity::new(&[3]);
/// let d = PartialComplexEntity::new(&[4]);
///
/// let ce1 = Instantiables::new(&[
///   a.clone(),
///   a.clone() & b.clone(),
///   a.clone() & c.clone(),
///   a.clone() & b.clone() & d.clone(),
///   b.clone() & c.clone(),
///   d.clone()
/// ]);
///
/// let ce2 = Instantiables::new(&[
///   b.clone(),
///   d.clone()
/// ]);
///
/// assert_eq!(ce1 / ce2, Instantiables::new(&[
///   a.clone() & b.clone(),
///   a.clone() & b.clone() & d.clone(),
///   b.clone() & c.clone(),
///   d.clone()
/// ]));
/// ```
///
/// - $[A_1, A_2, B_1, B_2] − [A_2, B_1] = [A_1, B_2]$
///
/// ```
/// # use espr::ir::*;
/// let a1 = PartialComplexEntity::new(&[1]);
/// let a2 = PartialComplexEntity::new(&[2]);
/// let b1 = PartialComplexEntity::new(&[3]);
/// let b2 = PartialComplexEntity::new(&[4]);
///
/// let ce1 = Instantiables::new(&[
///   a1.clone(),
///   a2.clone(),
///   b1.clone(),
///   b2.clone(),
/// ]);
///
/// let ce2 = Instantiables::new(&[
///   a2.clone(),
///   b1.clone()
/// ]);
///
/// assert_eq!(ce1 - ce2, Instantiables::new(&[
///   a1.clone(),
///   b2.clone()
/// ]));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Instantiables {
    /// Sorted and non-duplicated list of partial complex entities
    pub parts: Vec<PartialComplexEntity>,
}

impl Instantiables {
    pub fn new(pces: &[PartialComplexEntity]) -> Self {
        Self {
            parts: pces.to_vec(),
        }
    }

    /// Create from single index
    pub fn single(index: usize) -> Self {
        Self {
            parts: vec![PartialComplexEntity::new(&[index])],
        }
    }

    /// ONEOF(A, B, C) -> [A, B, C]
    pub fn oneof(parts: Vec<Self>) -> Self {
        let mut is = Self::default();
        for p in parts {
            is = is + p;
        }
        is
    }

    /// A AND B AND C -> [A & B & C]
    pub fn and(terms: Vec<Self>) -> Self {
        assert!(terms.len() >= 2);
        let mut constrait = None;
        for c in terms {
            constrait = Some(if let Some(constrait) = constrait {
                constrait & c
            } else {
                c
            });
        }
        constrait.unwrap()
    }

    /// A ANDOR B ANDOR C -> [A, B, C, A & B, B & C, A & C, A & B & C]
    pub fn andor(factors: Vec<Self>) -> Self {
        assert!(!factors.is_empty());
        // A ANDOR B → [A, B, A & B]
        //
        // This means `ANDOR` of n-factors will produce $2^n-1$ terms like:
        //
        // | A | B | ANDOR |
        // |---|---|-------|
        // | + | - | A     |
        // | - | + | B     |
        // | + | + | A & B |
        //
        let n = factors.len() as u32;
        let mut constrait = Self::default();
        for mut i in 1..(2usize.pow(n)) {
            // i=0b01 -> A
            // i=0b10 -> B
            // i=0b11 -> A & B, and so on.
            let mut c: Option<Self> = None;
            for factor in &factors {
                if i % 2 == 1 {
                    c = Some(if let Some(pre) = c {
                        pre & factor.clone()
                    } else {
                        factor.clone()
                    });
                }
                i >>= 1;
            }
            constrait = constrait + c.unwrap();
        }
        constrait
    }

    pub fn from_constraint_expr(
        ns: &Namespace,
        expr: &ConstraintExpr,
    ) -> Result<Self, SemanticError> {
        use ConstraintExpr::*;
        match expr {
            Reference(path) => {
                let (_ast, index) = ns.get(path)?;
                Ok(Self::single(index))
            }
            OneOf(exprs) => {
                let exprs = exprs
                    .iter()
                    .map(|e| Self::from_constraint_expr(ns, e))
                    .collect::<Result<Vec<Self>, SemanticError>>()?;
                Ok(Self::oneof(exprs))
            }
            And(exprs) => {
                let exprs = exprs
                    .iter()
                    .map(|e| Self::from_constraint_expr(ns, e))
                    .collect::<Result<Vec<Self>, SemanticError>>()?;
                Ok(Self::and(exprs))
            }
            AndOr(exprs) => {
                let exprs = exprs
                    .iter()
                    .map(|e| Self::from_constraint_expr(ns, e))
                    .collect::<Result<Vec<Self>, SemanticError>>()?;
                Ok(Self::andor(exprs))
            }
        }
    }

    /// Restore Path from namespace index
    pub fn as_path(&self, ns: &Namespace) -> Vec<Vec<Path>> {
        self.parts.iter().map(|pce| pce.as_path(ns)).collect()
    }
}

impl<'a> FromIterator<&'a PartialComplexEntity> for Instantiables {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a PartialComplexEntity>,
    {
        Self {
            parts: iter.into_iter().cloned().sorted().dedup().collect(),
        }
    }
}

// [A, B] + [C, D] = [A, B, C, D]
impl std::ops::Add for Instantiables {
    type Output = Self;
    fn add(mut self, mut rhs: Instantiables) -> Self {
        self.parts.append(&mut rhs.parts);
        Self {
            parts: self.parts.into_iter().sorted().dedup().collect(),
        }
    }
}

// [A, B] + C = [A, B, C]
impl std::ops::Add<PartialComplexEntity> for Instantiables {
    type Output = Self;
    fn add(mut self, rhs: PartialComplexEntity) -> Self {
        self.parts.push(rhs);
        Self {
            parts: self.parts.into_iter().sorted().dedup().collect(),
        }
    }
}

// A + [B, C] = [A, B, C]
impl std::ops::Add<Instantiables> for PartialComplexEntity {
    type Output = Instantiables;
    fn add(self, rhs: Instantiables) -> Instantiables {
        rhs + self
    }
}

// [A, B] & [C, D] = [A & C, A & D, B & C, B & D]
impl std::ops::BitAnd for Instantiables {
    type Output = Instantiables;
    fn bitand(self, rhs: Instantiables) -> Instantiables {
        let mut parts = Vec::with_capacity(self.parts.len() * rhs.parts.len());
        for p in &self.parts {
            for q in &rhs.parts {
                parts.push(p.clone() & q.clone());
            }
        }
        Instantiables {
            parts: parts.into_iter().sorted().dedup().collect(),
        }
    }
}

// [A, B] & C = [A & C, B & C]
impl std::ops::BitAnd<PartialComplexEntity> for Instantiables {
    type Output = Instantiables;
    fn bitand(self, q: PartialComplexEntity) -> Instantiables {
        Instantiables {
            parts: self
                .parts
                .into_iter()
                .map(|p| p & q.clone())
                .sorted()
                .dedup()
                .collect(),
        }
    }
}

// A & [B, C] = [A & B, A & C]
impl std::ops::BitAnd<Instantiables> for PartialComplexEntity {
    type Output = Instantiables;
    fn bitand(self, rhs: Instantiables) -> Instantiables {
        rhs & self
    }
}

impl std::ops::Sub for Instantiables {
    type Output = Self;
    fn sub(self, rhs: Instantiables) -> Self {
        Instantiables {
            parts: self
                .parts
                .into_iter()
                .filter(|p| rhs.parts.iter().all(|q| p != q))
                .collect(),
        }
    }
}

impl std::ops::Sub<PartialComplexEntity> for Instantiables {
    type Output = Self;
    fn sub(self, q: PartialComplexEntity) -> Self {
        Instantiables {
            parts: self.parts.into_iter().filter(|p| p != &q).collect(),
        }
    }
}

// [A, A & B, A & C, A & B & D, B & C, D]/[B, D] ≡ [A & B, A & B & D, B & C, D]
impl std::ops::Div for Instantiables {
    type Output = Self;
    fn div(self, rhs: Instantiables) -> Self {
        Instantiables {
            parts: self
                .parts
                .into_iter()
                .filter(|p| {
                    for q in &rhs.parts {
                        if q.indices.iter().all(|j| p.indices.binary_search(j).is_ok()) {
                            return true;
                        }
                    }
                    false
                })
                .collect(),
        }
    }
}

// [A, A & B, A & C, A & B & D, B & C, D] / A = [A, A & B, A & C, A & B & D]
impl std::ops::Div<PartialComplexEntity> for Instantiables {
    type Output = Instantiables;
    fn div(self, rhs: PartialComplexEntity) -> Instantiables {
        Instantiables {
            parts: self
                .parts
                .into_iter()
                .filter(|part| {
                    rhs.indices
                        .iter()
                        .all(|i| part.indices.binary_search(i).is_ok())
                })
                .collect(),
        }
    }
}
