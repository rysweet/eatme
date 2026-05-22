use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use eatme_core::ast::{Function, Parameter, Procedure, Program, Statement};
use roxmltree::Node;
use zip::write::SimpleFileOptions;

pub fn write_structured_a3p(name: &str, xml: &str) -> PathBuf {
    let root = std::env::current_dir()
        .expect("current dir")
        .join("target/test-work/structured-a3p");
    fs::create_dir_all(&root).expect("create structured a3p dir");

    let path = root.join(format!("{name}.a3p"));
    let file = fs::File::create(&path).unwrap_or_else(|err| {
        panic!("create {}: {err}", path.display());
    });
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("program.xml", SimpleFileOptions::default())
        .expect("start program.xml entry");
    writer.write_all(xml.as_bytes()).expect("write program.xml");
    writer.finish().expect("finish a3p archive");
    path
}

pub fn parse_structured_a3p_program(path: &Path) -> Option<Program> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).ok()?;
        if !entry.name().ends_with(".xml") {
            continue;
        }

        let mut xml = String::new();
        entry.read_to_string(&mut xml).ok()?;
        if let Some(program) = parse_structured_program_xml(&xml) {
            return Some(program);
        }
    }

    None
}

fn parse_structured_program_xml(xml: &str) -> Option<Program> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    let root = doc.root_element();
    if !matches!(root.tag_name().name(), "program" | "eatme-program") {
        return None;
    }

    let procedures = root
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "procedure")
        .map(parse_procedure)
        .collect::<Option<Vec<_>>>()?;
    let functions = root
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "function")
        .map(parse_function)
        .collect::<Option<Vec<_>>>()?;

    if procedures.is_empty() && functions.is_empty() {
        return None;
    }

    Some(Program {
        procedures,
        functions,
    })
}

fn parse_procedure(node: Node<'_, '_>) -> Option<Procedure> {
    Some(Procedure {
        name: node.attribute("name")?.into(),
        parameters: parse_parameters(node),
        body: parse_named_block(node, "body"),
    })
}

fn parse_function(node: Node<'_, '_>) -> Option<Function> {
    Some(Function {
        name: node.attribute("name")?.into(),
        return_type: node.attribute("return_type").unwrap_or("Object").into(),
        body: parse_named_block(node, "body"),
    })
}

fn parse_parameters(node: Node<'_, '_>) -> Vec<Parameter> {
    node.children()
        .filter(|child| child.is_element() && child.tag_name().name() == "parameter")
        .filter_map(|parameter| {
            Some(Parameter {
                name: parameter.attribute("name")?.into(),
                param_type: parameter.attribute("type").unwrap_or("Object").into(),
            })
        })
        .collect()
}

fn parse_named_block(node: Node<'_, '_>, block_name: &str) -> Vec<Statement> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == block_name)
        .map(parse_statement_children)
        .unwrap_or_default()
}

fn parse_statement_children(node: Node<'_, '_>) -> Vec<Statement> {
    node.children()
        .filter(|child| child.is_element() && child.tag_name().name() == "statement")
        .filter_map(parse_statement)
        .collect()
}

fn parse_statement(node: Node<'_, '_>) -> Option<Statement> {
    match node.attribute("type")? {
        "MethodInvocation" => Some(Statement::MethodCall {
            object: node.attribute("object").unwrap_or("this").into(),
            method: node.attribute("method")?.into(),
            arguments: parse_arguments(node),
        }),
        "CountLoop" => Some(Statement::CountLoop {
            count: node.attribute("count").unwrap_or("1").parse().unwrap_or(1),
            body: parse_named_block(node, "body"),
        }),
        "ConditionalStatement" => Some(Statement::IfElse {
            condition: node.attribute("condition").unwrap_or("").into(),
            if_body: parse_named_block(node, "ifBody"),
            else_body: parse_named_block(node, "elseBody"),
        }),
        "AddEventListener" => Some(Statement::EventListener {
            event: node.attribute("event").unwrap_or("SceneActivated").into(),
            body: parse_named_block(node, "body"),
        }),
        "CollisionStartListener" | "CollisionStartEventListener" => {
            Some(Statement::CollisionListener {
                object_a: node.attribute("object_a").unwrap_or("unknown").into(),
                object_b: node.attribute("object_b").unwrap_or("unknown").into(),
                body: parse_named_block(node, "body"),
            })
        }
        "DoInOrder" => Some(Statement::DoInOrder {
            body: parse_named_block(node, "body"),
        }),
        "ReturnStatement" => Some(Statement::ReturnStatement {
            expression: node.attribute("expression").unwrap_or("").into(),
        }),
        "FunctionCall" => Some(Statement::FunctionCall {
            object: node.attribute("object").unwrap_or("this").into(),
            function: node.attribute("function")?.into(),
            arguments: parse_arguments(node),
        }),
        "VariableDeclaration" => Some(Statement::VariableDeclaration {
            name: node.attribute("name")?.into(),
            var_type: node.attribute("var_type").unwrap_or("Object").into(),
            initial_value: node.attribute("initial_value").unwrap_or("").into(),
        }),
        "VariableAssignment" => Some(Statement::VariableAssignment {
            name: node.attribute("name")?.into(),
            value: node.attribute("value").unwrap_or("").into(),
        }),
        _ => None,
    }
}

fn parse_arguments(node: Node<'_, '_>) -> Vec<String> {
    let mut arguments: Vec<String> = node
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "argument")
        .filter_map(|argument| {
            argument
                .attribute("value")
                .or_else(|| argument.text())
                .map(str::to_string)
        })
        .collect();

    if arguments.is_empty()
        && let Some(raw) = node.attribute("arguments")
    {
        arguments = raw
            .split('|')
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }

    arguments
}
