use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub procedures: Vec<Procedure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<Function>,
}

impl Program {
    pub fn new(procedures: Vec<Procedure>) -> Self {
        Self {
            procedures,
            functions: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub body: Vec<Statement>,
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
    ReturnStatement {
        expression: String,
    },
    FunctionCall {
        object: String,
        function: String,
        arguments: Vec<String>,
    },
}

#[cfg(test)]
#[path = "ast_tests.rs"]
mod tests;
