//! ruststep is a crate for reading and writing ASCII encoding of exchange structure,
//! a.k.a. STEP file, and for mapping them into Rust structs generated by [espr](../espr/index.html) compiler.
//!
//! This crate also contains generated code as sub-modules for following schemas:
//!
//! - [ISO 10303-201 "Part 201: Application protocol: Explicit draughting"](https://www.iso.org/standard/20595.html)
//! - [ISO 10303-203 "Part 203: Application protocol: Configuration controlled 3D design of mechanical parts and assemblies"](https://www.iso.org/standard/44305.html)
//!
//! They are compiled only when the `features` are enabled in cargo to keep compile faster:
//!
//! ```toml
//! [dependencies]
//! ruststep = { version = "*", features = ["ap201", "ap203"] }
//! ```
//!
//! These features are not default.
//!
//! ASCII encoding of exchange structure
//! -------------------------------------
//!
//! ASCII encoding of exchange structure is defined in
//! [ISO-10303-21 "Part 21: Implementation methods: Clear text encoding of the exchange structure"](https://www.iso.org/standard/63141.html).
//! This ISO document contains an example in section Annex H "Example of a complete exchange structure":
//!
//! ```text
//! ISO-10303-21; /* start exchange structure */
//!
//! HEADER; /* start header section */
//!
//!   FILE_DESCRIPTION(
//!     ('THIS FILE CONTAINS A SMALL SAMPLE STEP MODEL'),
//!     '3;1'
//!   );
//!
//!   FILE_NAME(
//!     'EXAMPLE STEP FILE #1',
//!     '2013-02-11T15:30:00',
//!     ('JOHN DOE', 'ACME INC.', 'METROPOLIS USA'),
//!     ('ACME INC. A SUBSIDIARY OF GIANT INDUSTRIES', 'METROPOLIS USA'),
//!     'CIM/STEP VERSION2',
//!     'SUPER CIM SYSTEM RELEASE 4.0',
//!     'APPROVED BY JOE BLOGGS'
//!   );
//!
//!   FILE_SCHEMA(('EXAMPLE_GEOMETRY'));
//!
//! ENDSEC; /* end header section */
//!
//! DATA; /* start data section */
//!
//!   /* The following 13 entities represent a triangular edge loop */
//!
//!   /* cartesian point entity */
//!   #1 = CPT(0.0, 0.0, 0.0);
//!   #2 = CPT(0.0, 1.0, 0.0);
//!   #3 = CPT(1.0, 0.0, 0.0);
//!
//!   /* vertex entity */
//!   #11 = VX(#1);
//!   #12 = VX(#2);
//!   #13 = VX(#3);
//!
//!   /* edge entity */
//!   #16 = ED(#11, #12);
//!   #17 = ED(#11, #13);
//!   #18 = ED(#13, #12);
//!
//!   /* edge logical structure entity */
//!   #21 = ED_STRC(#17, .F.);
//!   #22 = ED_STRC(#18, .F.);
//!   #23 = ED_STRC(#16, .T.);
//!
//!   /* edge loop entity */
//!   #24 = ED_LOOP((#21, #22, #23));
//!
//! ENDSEC; /* end data section */
//!
//! END-ISO-10303-21; /* end exchange structure */
//! ```
//!
//! Spaces, indent, and comments are modified for better understanding.
//! This tells us what consists of exchange structure:
//!
//! - It starts with `ISO-10303-21;` and ends with `END-ISO-10303-21;`.
//! - It contains `HEADER` and `DATA` sections.
//! - `HEADER` section has three components `FILE_DESCRIPTION`, `FILE_NAME`, and `FILE_SCHEMA`.
//!   - See [header] module document for detail.
//! - Each data is in form `TYPE_NAME(parameter1, ...)`.
//!   - This is called "Record".
//!   - Each records is bounded by a number.
//!   - Parameter can be
//!     - Floating number, e.g. `0.0`
//!     - String, e.g. `'EXAMPLE STRING'`
//!     - List, e.g. `(1.0, 2.0)`
//!     - Enum value, e.g. `.T.` (means true), `.F.` (means false)
//!     - Reference, e.g. `#1`
//! - Actual data, triangular geometry in this example, is stored in `DATA` section.
//!
//! See the module document of [parser] for detail.
//!
//! XML interoperation
//! -------------------
//! STEP implementation using XML(eXtensible Markup Language) is defined in
//! [ISO-10303-28](https://www.iso.org/standard/40646.html).
//!
//! Not supported yet. See [tracking issue](https://github.com/ricosjp/ruststep/issues/215).
//!

#![deny(rustdoc::broken_intra_doc_links)]

pub mod ast;
pub mod error;
pub mod header;
pub mod parser;
pub mod primitive;
pub mod tables;

// To work generated code by ruststep-derive only with ruststep
pub use derive_more;
pub use itertools;
pub use serde;

pub use ruststep_derive::*;

// Automatically generated codes
#[cfg(feature = "ap201")]
pub mod ap201;
#[cfg(feature = "ap203")]
pub mod ap203;
