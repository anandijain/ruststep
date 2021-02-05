use super::{entity::*, expression::*, identifier::*, stmt::*, subsuper::*, types::*, util::*};

/// Parsed result of EXPRESS's SCHEMA
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub name: String,
    pub entities: Vec<Entity>,
    pub types: Vec<TypeDecl>,
}

/// 296 schema_decl = SCHEMA [schema_id] \[ schema_version_id \] `;` [schema_body] END_SCHEMA `;` .
pub fn schema_decl(input: &str) -> ParseResult<Schema> {
    // FIXME schema_version_id
    let schema_head =
        tuple((tag("SCHEMA "), schema_id, char(';'))).map(|(_start, id, _semicoron)| id);
    tuple((schema_head, schema_body, tag("END_SCHEMA"), char(';')))
        .map(|(name, (entities, types), _end, _semicoron)| Schema {
            name,
            entities,
            types,
        })
        .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Entity(Entity),
    Type(TypeDecl),
    Function(Function),
    Procedure(Procedure),
}

/// 199 declaration = [entity_decl] | [function_decl] | [procedure_decl] | [subtype_constraint_decl] | [type_decl] .
pub fn declaration(input: &str) -> ParseResult<Declaration> {
    // FIXME subtype_constraint_decl
    alt((
        entity_decl.map(|e| Declaration::Entity(e)),
        type_decl.map(|ty| Declaration::Type(ty)),
        function_decl.map(|f| Declaration::Function(f)),
        procedure_decl.map(|p| Declaration::Procedure(p)),
    ))
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub name: String,
    pub parameters: Vec<FormalParameter>,
    pub declarations: Vec<Declaration>,
    pub constants: Vec<Constant>,
    pub variables: Vec<LocalVariable>,
    pub statements: Vec<Statement>,
}

/// 271 procedure_decl = [procedure_head] [algorithm_head] { [stmt] } END_PROCEDURE `;` .
pub fn procedure_decl(input: &str) -> ParseResult<Procedure> {
    tuple((
        procedure_head,
        algorithm_head,
        spaced_many0(stmt),
        tag("END_PROCEDURE"),
        char(';'),
    ))
    .map(
        |(
            (name, parameters),
            (declarations, constants, variables),
            statements,
            _end,
            _semicolon,
        )| Procedure {
            name,
            parameters,
            declarations,
            constants,
            variables,
            statements,
        },
    )
    .parse(input)
}

/// 272 procedure_head = PROCEDURE [procedure_id]
///                    \[ `(`
///                      \[ VAR \] [formal_parameter] { `;`
///                      \[ VAR \] [formal_parameter]
///                    }
///                    `)` \] `;` .
pub fn procedure_head(input: &str) -> ParseResult<(String, Vec<FormalParameter>)> {
    let param = tuple((opt(tag("VAR")), formal_parameter)).map(|(var, mut params)| {
        for mut param in &mut params {
            param.is_variable = var.is_some();
        }
        params
    });
    tuple((
        tag("PROCEDURE"),
        procedure_id,
        opt(
            tuple((char('('), semicolon_separated(param), char(')'))).map(
                |(_open, params, _close)| {
                    params
                        .into_iter()
                        .map(|ps| ps.into_iter())
                        .flatten()
                        .collect()
                },
            ),
        )
        .map(|opt| opt.unwrap_or(Vec::new())),
        char(';'),
    ))
    .map(|(_procedure, name, parameters, _semicolon)| (name, parameters))
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<FormalParameter>,
    pub declarations: Vec<Declaration>,
    pub constants: Vec<Constant>,
    pub variables: Vec<LocalVariable>,
    pub statements: Vec<Statement>,
    pub return_type: ParameterType,
}

/// 220 function_decl = [function_head] [algorithm_head] [stmt] { [stmt] } END_FUNCTION `;` .
pub fn function_decl(input: &str) -> ParseResult<Function> {
    tuple((
        function_head,
        algorithm_head,
        space_separated(stmt),
        tag("END_FUNCTION"),
        char(';'),
    ))
    .map(
        |(
            (name, parameters, return_type),
            (declarations, constants, variables),
            statements,
            _end,
            _semicoron,
        )| Function {
            name,
            parameters,
            declarations,
            constants,
            variables,
            statements,
            return_type,
        },
    )
    .parse(input)
}

/// 221 function_head = FUNCTION [function_id]
///                   \[ `(` [formal_parameter] { `;` [formal_parameter] } `)` \]
///                   `:` [parameter_type] `;` .
pub fn function_head(input: &str) -> ParseResult<(String, Vec<FormalParameter>, ParameterType)> {
    tuple((
        tag("FUNCTION"),
        function_id,
        opt(
            tuple((char('('), space_separated(formal_parameter), char(')'))).map(
                |(_open, params, _close)| {
                    params
                        .into_iter()
                        .map(|ps| ps.into_iter())
                        .flatten()
                        .collect()
                },
            ),
        )
        .map(|opt| opt.unwrap_or(Vec::new())),
        char(':'),
        parameter_type,
        char(';'),
    ))
    .map(|(_function, name, parameters, _comma, ty, _semicoron)| (name, parameters, ty))
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormalParameter {
    pub name: String,
    pub ty: ParameterType,
    /// `true` if specified with `VAR` in `PROCEDURE`. Always `false` for `FUNCTION`
    pub is_variable: bool,
}

/// 218 formal_parameter = [parameter_id] { `,` [parameter_id] } `:` [parameter_type] .
pub fn formal_parameter(input: &str) -> ParseResult<Vec<FormalParameter>> {
    tuple((comma_separated(parameter_id), char(':'), parameter_type))
        .map(|(names, _comma, ty)| {
            names
                .into_iter()
                .map(|name| FormalParameter {
                    name,
                    ty: ty.clone(),
                    is_variable: false,
                })
                .collect()
        })
        .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constant {
    pub name: String,
    pub ty: ConcreteType,
    pub expr: Expression,
}

/// 195 constant_decl = CONSTANT [constant_body] { [constant_body] } END_CONSTANT `;` .
pub fn constant_decl(input: &str) -> ParseResult<Vec<Constant>> {
    tuple((
        tag("CONSTANT"),
        space_separated(constant_body),
        tag("END_CONSTANT"),
        char(';'),
    ))
    .map(|(_constant, consts, _end, _semicoron)| consts)
    .parse(input)
}

/// 194 constant_body = [constant_id] `:` [instantiable_type] `:=` [expression] `;` .
pub fn constant_body(input: &str) -> ParseResult<Constant> {
    tuple((
        constant_id,
        char(':'),
        instantiable_type,
        tag(":="),
        expression,
        char(';'),
    ))
    .map(|(name, _coron, ty, _def, expr, _semicoron)| Constant { name, ty, expr })
    .parse(input)
}

/// 240 instantiable_type = [concrete_types] | [entity_ref] .
pub fn instantiable_type(input: &str) -> ParseResult<ConcreteType> {
    alt((
        concrete_types,
        entity_ref.map(|r| ConcreteType::Reference(r)),
    ))
    .parse(input)
}

/// 291 rule_decl = [rule_head] [algorithm_head] { [stmt] } [where_clause] END_RULE `;` .
pub fn rule_decl(input: &str) -> ParseResult<()> {
    todo!()
}

/// 292 rule_head = RULE [rule_id] FOR `(` [entity_ref] { `,` [entity_ref] } `)` `;` .
pub fn rule_head(input: &str) -> ParseResult<()> {
    todo!()
}

/// 173 algorithm_head = { [declaration] } \[ [constant_decl] \] \[ [local_decl] \] .
pub fn algorithm_head(
    input: &str,
) -> ParseResult<(Vec<Declaration>, Vec<Constant>, Vec<LocalVariable>)> {
    tuple((
        spaced_many0(declaration),
        opt(constant_decl).map(|opt| opt.unwrap_or(Vec::new())),
        opt(local_decl).map(|opt| opt.unwrap_or(Vec::new())),
    ))
    .parse(input)
}

/// 252 local_decl = LOCAL [local_variable] { [local_variable] } END_LOCAL `;` .
pub fn local_decl(input: &str) -> ParseResult<Vec<LocalVariable>> {
    tuple((
        tag("LOCAL"),
        space_separated(local_variable),
        tag("END_LOCAL"),
        char(';'),
    ))
    .map(|(_local, vars, _end, _semicoron)| {
        vars.into_iter()
            .map(|var| var.into_iter())
            .flatten()
            .collect()
    })
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariable {
    pub name: String,
    pub ty: ParameterType,
    pub expr: Option<Expression>,
}

/// 253 local_variable = [variable_id] { `,` [variable_id] } `:` [parameter_type] \[ `:=` [expression] \] `;` .
pub fn local_variable(input: &str) -> ParseResult<Vec<LocalVariable>> {
    tuple((
        comma_separated(variable_id),
        char(':'),
        parameter_type,
        opt(tuple((tag(":="), expression)).map(|(_def, expr)| expr)),
        char(';'),
    ))
    .map(|(names, _comma, ty, expr, _semicoron)| {
        names
            .into_iter()
            .map(|name| LocalVariable {
                name,
                ty: ty.clone(),
                expr: expr.clone(),
            })
            .collect()
    })
    .parse(input)
}

/// 242 interface_specification = [reference_clause] | [use_clause] .
pub fn interface_specification(input: &str) -> ParseResult<()> {
    todo!()
}

/// 281 reference_clause = REFERENCE FROM [schema_ref] \[ `(` [resource_or_rename] { `,` [resource_or_rename] } `)` \] `;` .
pub fn reference_clause(input: &str) -> ParseResult<()> {
    todo!()
}

/// 288 resource_or_rename = [resource_ref] \[ AS [rename_id] \] .
pub fn resource_or_rename(input: &str) -> ParseResult<()> {
    todo!()
}

/// 336 use_clause = USE FROM [schema_ref] \[ `(` [named_type_or_rename] { `,` [named_type_or_rename] } `)` \] `;` .
pub fn use_clause(input: &str) -> ParseResult<()> {
    todo!()
}

/// 259 named_type_or_rename = [named_types] \[ AS ( [entity_id] | [type_id] ) \] .
pub fn named_type_or_rename(input: &str) -> ParseResult<()> {
    todo!()
}

/// 295 schema_body = { [interface_specification] } \[ [constant_decl] \] { [declaration] | [rule_decl] } .
pub fn schema_body(input: &str) -> ParseResult<(Vec<Entity>, Vec<TypeDecl>)> {
    // FIXME interface_specification
    // FIXME constant_decl
    // FIXME rule_decl
    spaced_many0(declaration)
        .map(|decls| {
            let mut entities = Vec::new();
            let mut types = Vec::new();
            for decl in decls {
                match decl {
                    Declaration::Entity(e) => entities.push(e),
                    Declaration::Type(ty) => types.push(ty),
                }
            }
            (entities, types)
        })
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Finish;

    #[test]
    fn schema() {
        let exp_str = r#"
        SCHEMA my_first_schema;
          ENTITY first;
            m_ref : second;
            fattr : STRING;
          END_ENTITY;

          ENTITY second;
            sattr : STRING;
          END_ENTITY;
        END_SCHEMA;
        "#
        .trim();

        let (residual, (schema, _remark)) = super::schema_decl(exp_str).finish().unwrap();
        assert_eq!(schema.name, "my_first_schema");
        assert_eq!(schema.entities.len(), 2);
        assert_eq!(
            schema.entities[0],
            entity_decl(
                r#"
                ENTITY first;
                  m_ref : second;
                  fattr : STRING;
                END_ENTITY;
                "#
                .trim()
            )
            .finish()
            .unwrap()
            .1
             .0
        );
        assert_eq!(
            schema.entities[1],
            entity_decl(
                r#"
                ENTITY second;
                  sattr : STRING;
                END_ENTITY;
                "#
                .trim()
            )
            .finish()
            .unwrap()
            .1
             .0
        );
        assert_eq!(residual, "");
    }
}
