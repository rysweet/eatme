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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
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
    VariableDeclaration {
        name: String,
        var_type: String,
        initial_value: String,
    },
    VariableAssignment {
        name: String,
        value: String,
    },
    ArrayDeclaration {
        name: String,
        element_type: String,
        elements: Vec<String>,
    },
    ArrayAccess {
        array: String,
        index: String,
        target: String,
    },
    ForEachArray {
        item_name: String,
        array: String,
        body: Vec<Statement>,
    },
    ArithmeticExpression {
        operator: ArithmeticOperator,
        left: String,
        right: String,
        result: String,
    },
    Comment {
        text: String,
    },
    UserTypeDeclaration {
        name: String,
        extends: Option<String>,
        methods: Vec<Procedure>,
    },
}

#[cfg(test)]
#[path = "ast_tests.rs"]
mod tests;
