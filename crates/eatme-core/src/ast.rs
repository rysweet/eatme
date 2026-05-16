use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub procedures: Vec<Procedure>,
    #[serde(default)]
    pub functions: Vec<Function>,
    #[serde(default)]
    pub variable_declarations: Vec<VariableDeclaration>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: String,
    pub var_type: String,
    pub initial_value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Statement {
    MethodCall {
        object: String,
        method: String,
        arguments: Vec<String>,
    },
    CountLoop {
        count: u32,
        body: Vec<Statement>,
    },
    IfElse {
        condition: String,
        if_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    EventListener {
        event: String,
        body: Vec<Statement>,
    },
    CollisionListener {
        object_a: String,
        object_b: String,
        body: Vec<Statement>,
    },
    FunctionCall {
        function_name: String,
    },
    ReturnStatement {
        value: String,
    },
    VariableAssignment {
        variable: String,
        value: String,
    },
}

#[cfg(test)]
#[path = "ast_tests.rs"]
mod tests;
