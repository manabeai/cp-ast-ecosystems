/// Expected type for `TypeDecl` constraints.
///
/// Rev.1: Simplified to 3 variants. Array/Tuple/Float removed.
/// Complex type info is expressed via constraint composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedType {
    Int,
    Str,
    Char,
}
