use tree_sitter::{Language, Parser, Query, QueryCursor};

use clap::{arg, value_parser, Command};
use ignore::Walk;
use std::{fs, path::PathBuf};

extern "C" {
    fn tree_sitter_erlang() -> Language;
}

fn main() {
    println!("Olá!");

    let matches = Command::new("scryer")
        .arg(
            arg!(-r --root <ROOT> "Path of the root of the project")
                .default_value(".")
                // .value_parser(value_parser!(PathBuf))
        )
        .arg(arg!(
            -q --query <QUERY> "Tree-sitter query"
        )
             .required(true))
        .get_matches();

    let root = matches.get_one::<String>("root").unwrap();
    let query_source = matches.get_one::<String>("query").unwrap();

    let lang = unsafe { tree_sitter_erlang() };
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    for r in Walk::new(root) {
        if let Ok(r) = r {
            let p = r.path();
            match p.extension() {
                Some(e) if e == "erl" => {
                    let source = fs::read_to_string(p).unwrap();
                    let tree = parser.parse(&source, None).unwrap();
                    let root = tree.root_node();
                    println!("whole tree:\n  {}", root.to_sexp());
                    let query = Query::new(lang, query_source).unwrap();
                    let mut cursor = QueryCursor::new();
                    for m in cursor.matches(&query, root, source.as_bytes()) {
                        println!("query match: {:?}", m);
                        for cap in m.captures {
                            println!("capture: {:?}", cap);
                            println!("capture text: {:?}", cap.node.utf8_text(source.as_bytes()));
                        }
                    }
                },
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::{Parser, Query, QueryCursor, Point, TreeCursor};

    use crate::tree_sitter_erlang;

    #[test]
    fn test_parser() {
        let lang = unsafe { tree_sitter_erlang() };
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        let source = "-module(bah).\n-define(A, a).\nfoo(X) ->\n  {?A, X + 1}.";
        let tree = parser.parse(source, None).unwrap();
        assert_eq!(tree.root_node().to_sexp(), "(source_file forms: (module_attribute name: (atom)) forms: (pp_define lhs: (macro_lhs name: (var)) replacement: (atom)) forms: (fun_decl clauses: (function_clause name: (atom) args: (expr_args args: (var)) body: (clause_body exprs: (tuple expr: (macro_call_expr name: (var)) expr: (binary_op_expr lhs: (var) rhs: (integer)))))))");
    }

    #[test]
    fn test_to_text() {
        let lang = unsafe { tree_sitter_erlang() };
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        let source = "-module(bah).\n-define(A, a).\nfoo(X) ->\n  {?A, X + 1}.";
        let tree = parser.parse(source, None).unwrap();
        assert_eq!(
            tree.root_node()
                .child(2)
                .unwrap()
                .child_by_field_name("clauses")
                .unwrap()
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap(),
            "foo"
        );
    }

    #[test]
    fn test_query() {
        let lang = unsafe { tree_sitter_erlang() };
        let mut parser = Parser::new();
        parser.set_language(lang).unwrap();
        let source = "-module(bah).\n-define(A, a).\nfoo(X) ->\n  bah(1, x, 2),\n  bah(x, 2),\n  bah(1, x),\n  {?A, X + 1}.";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        println!("root sexp: {}\n\n", tree.root_node().to_sexp());
        let query_source = "(call args: (expr_args _* (atom)@atomo _*))";
        let query = Query::new(lang, query_source).unwrap();
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, root, source.as_bytes()) {
            println!("query match: {:?}", m);
            println!("query captures: {:?}", m.captures);
            for cap in m.captures {
                println!("capture: {:?}", cap);
                println!("capture text: {:?}", cap.node.utf8_text(source.as_bytes()));
                println!("capture parent: {:?}", cap.node.parent());
                println!("capture parent child count: {:?}", cap.node.parent().unwrap().child_count());
                let mut ccursor = cap.node.parent().unwrap().walk();
                let mut success = ccursor.goto_first_child();
                while success {
                    println!("child: {:?}", ccursor.node());
                    println!("child kind: {:?}", ccursor.node().kind());
                    success = ccursor.goto_next_sibling();
                }
            }
        }
        assert!(false);
        assert_eq!(
            vec![("x", (Point::new(10, 10), Point::new(10, 10)))],
            cursor
                .matches(&query, root, source.as_bytes())
                .into_iter()
                .flat_map(|m| {
                    m.captures
                        .into_iter()
                        .map(|c| {
                            let spos = c.node.start_position();
                            let epos = c.node.end_position();
                            let atom_name = c.node.utf8_text(source.as_bytes()).unwrap();
                            (atom_name, (spos, epos))
                        })
                })
                .collect::<Vec<_>>()
        );
    }
}
