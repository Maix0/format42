#![allow(dead_code)]
use std::io::Read;
use std::{collections::VecDeque, mem::MaybeUninit};

use std::cell::RefCell;

thread_local! {
    pub static CURRENT_DATA_BYTES: RefCell<Box<[u8]>> = RefCell::new(Vec::new().into_boxed_slice());
    pub static CURRENT_OUTPUT: RefCell<Vec<u8>> = const {RefCell::new(Vec::new())};
}
use tree_sitter::{Node, TextProvider, Tree};

const TEST_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "sample/", "1.c");

fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        let mut buffer = Vec::with_capacity(1024);
        std::io::stdin().lock().read_to_end(&mut buffer).unwrap();
        run(buffer.into_boxed_slice(), Vec::with_capacity(1024));
        return;
    }
    args.iter()
        .map(|o| std::fs::OpenOptions::new().read(true).write(true).open(o))
        .map(|res| res.map(|f| f.metadata().map(|m| (f, m))))
        .map(|res| match res {
            Err(e) | Ok(Err(e)) => Err(e),
            Ok(Ok(v)) => Ok(v),
        })
        .map(|res| {
            res.map(|(file, metadata)| (file, Vec::<u8>::with_capacity(metadata.len() as usize)))
        })
        .map(|res| -> Result<_, std::io::Error> {
            let (mut f, o) = res?;
            let mut buffer = Vec::with_capacity(o.len());
            f.read_to_end(&mut buffer)?;
            Ok((f, buffer, o))
        }) // .into_par_iter()
        .map(|res| res.map(|(f, i, o)| (f, run(i.into_boxed_slice(), o))))
        .for_each(|res| match res {
            Ok((f, o)) => {
                println!("{:?}: \n{:}", f, std::str::from_utf8(&o).unwrap());
            }
            Err(e) => {
                println!("Error: {e}");
            }
        });
}

fn run(data: Box<[u8]>, output: Vec<u8>) -> Vec<u8> {
    let mut ts = tree_sitter::Parser::new();
    ts.set_language(tree_sitter_c::language()).unwrap();
    if std::str::from_utf8(&data).is_err() {
        return Vec::new();
    }
    CURRENT_DATA_BYTES.set(data);
    let tree = ts.parse(get_data(&()), None).unwrap();
    let top_level = ToplevelDefinition::from_tree(&tree);

    dbg!(&top_level);
    output
}

#[derive(Debug, Clone)]
enum TopLevelBlock<'ts> {
    PreprocIf(PreprocIfData<'ts>, ToplevelDefinition<'ts>),
    Plain(ToplevelDefinition<'ts>),
    Error(Node<'ts>),
}

#[derive(Debug, Clone)]
struct PreprocIfData<'ts> {
    node: Node<'ts>,
    ifnode: Option<Node<'ts>>,
    ifnode_identifier: Option<Node<'ts>>,
    tlb: Vec<TopLevelBlock<'ts>>,
}

#[derive(Debug, Clone)]
struct ToplevelDefinition<'ts> {
    header: CommentBlock<'ts>,
    functions: FnDefinitionBlock<'ts>,
    declarations: DeclarationBlock<'ts>,
    includes: IncludeBlock<'ts>,
    leftovers_comments: CommentBlock<'ts>,
    defines: Vec<Define<'ts>>,
}

#[derive(Debug, Clone)]
struct Define<'ts>(Node<'ts>);

#[derive(Debug, Clone)]
struct CommentBlock<'ts>(Vec<Node<'ts>>);
#[derive(Debug, Clone)]
struct DeclarationBlock<'ts>(Vec<Declaration<'ts>>);
#[derive(Debug, Clone)]
struct Declaration<'ts>(CommentBlock<'ts>, Node<'ts>);

#[derive(Debug, Clone)]
struct FnDefinitionBlock<'ts>(Vec<FunctionDefinition<'ts>>);
#[derive(Debug, Clone)]
struct FunctionDefinition<'ts>(CommentBlock<'ts>, Node<'ts>);
#[derive(Debug, Clone)]
struct IncludeBlock<'ts>(CommentBlock<'ts>, Vec<Node<'ts>>);

impl<'ts> Default for CommentBlock<'ts> {
    fn default() -> Self {
        Self(Vec::with_capacity(4))
    }
}

fn get_data(_: &()) -> &'_ [u8] {
    unsafe { &*CURRENT_DATA_BYTES.with_borrow(|r| &**r as *const [u8]) }
}

impl<'ts> CommentBlock<'ts> {
    pub fn print(&self, ident_value: usize, fmt: &mut impl std::fmt::Write) -> std::fmt::Result {
        let mut data = get_data(&());
        let nodes = self.0.as_slice();
        let multi_line = nodes
            .iter()
            .map(|&n| data.text(n).next().unwrap())
            .map(|b| unsafe { std::str::from_utf8_unchecked(b) })
            .any(|s| s.starts_with("/*") || s.ends_with("*/"));
        let options = textwrap::Options::new(80 - 4 * ident_value)
            .initial_indent("//\t")
            .subsequent_indent("// \t");
        Ok(())
    }
}

impl<'ts> ToplevelDefinition<'ts> {
    fn from_tree_inner(root: &Node<'ts>, append_to: &mut Vec<TopLevelBlock<'ts>>, first: bool) {
        let mut walker = root.walk();
        let mut children = root.children(&mut walker).collect::<VecDeque<_>>();
        let mut out = ToplevelDefinition {
            header: CommentBlock(Vec::with_capacity(11)),
            declarations: DeclarationBlock(Vec::with_capacity(8)),
            functions: FnDefinitionBlock(Vec::with_capacity(5)),
            includes: IncludeBlock(CommentBlock(Vec::new()), Vec::with_capacity(4)),
            leftovers_comments: CommentBlock(Vec::new()),
            defines: Vec::new(), // Defines<'ts>>,
        };
        if first
            && children
                .iter()
                .take(11)
                .enumerate()
                .filter(|&(row, n)| {
                    n.kind() == "comment"
                        && n.end_position().column == 80
                        && n.start_position().column == 0
                        && n.start_position().row == row
                        && n.end_position().row == row
                })
                .count()
                == 11
        {
            let mut header_comments: [MaybeUninit<Node<'ts>>; 11] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for slot in &mut header_comments {
                slot.write(children.pop_front().unwrap());
            }
            out.header.0.extend(
                header_comments
                    .into_iter()
                    .map(|mu| unsafe { MaybeUninit::assume_init(mu) }),
            );
        }

        let mut latest_comment_block = CommentBlock(Vec::with_capacity(4));
        let mut ifdata: Option<Node<'ts>> = None;
        let mut ifdata_identifier: Option<Node<'ts>> = None;
        let mut inner_stuff = Vec::new();

        while let Some(node) = children.pop_front() {
            match node.kind() {
                "\n" => (),
                "ERROR" => {
                    append_to.push(TopLevelBlock::Error(node));
                    return;
                }
                "comment" => latest_comment_block.0.push(node),
                "function_definition" => out.functions.0.push(FunctionDefinition(
                    std::mem::take(&mut latest_comment_block),
                    node,
                )),
                "declaration" => out
                    .declarations
                    .0
                    .push(Declaration(std::mem::take(&mut latest_comment_block), node)),
                "preproc_ifdef" | "preproc_if" => Self::from_tree_inner(
                    &node,
                    if first { append_to } else { &mut inner_stuff },
                    false,
                ),
                "preproc_include" => {
                    out.includes.1.push(node);
                    (out.includes.0).0.append(&mut latest_comment_block.0);
                }
                "preproc_def" => {
                    out.defines.push(Define(node));
                }

                "#if" if !first => {
                    ifdata = Some(node);
                    ifdata_identifier = children
                        .front()
                        .map(|n| n.kind() == "binary_expression")
                        .unwrap_or_default()
                        .then(|| children.pop_front())
                        .flatten();
                }
                "#ifndef" | "#ifdef" if !first => {
                    ifdata = Some(node);
                    ifdata_identifier = children
                        .front()
                        .map(|n| n.kind() == "identifier")
                        .unwrap_or_default()
                        .then(|| children.pop_front())
                        .flatten();
                }
                "#endif" if !first => (),
                unknown => eprintln!(
                    "Unknown Node type `{unknown}`\n\tText: '{:?}'",
                    node.utf8_text(get_data(&())).unwrap()
                ),
            }
        }

        out.leftovers_comments = latest_comment_block;
        append_to.push(if first {
            TopLevelBlock::Plain(out)
        } else {
            TopLevelBlock::PreprocIf(
                PreprocIfData::from_node(*root, ifdata, ifdata_identifier, inner_stuff),
                out,
            )
        });
    }
    pub fn from_tree(root: &'ts Tree) -> Vec<TopLevelBlock<'ts>> {
        let root_node = root.root_node();
        let mut out_vec = Vec::with_capacity(2);
        Self::from_tree_inner(&root_node, &mut out_vec, true);
        out_vec
    }
}

impl<'ts> PreprocIfData<'ts> {
    fn from_node(
        node: Node<'ts>,
        ifnode: Option<Node<'ts>>,
        ifnode_identifier: Option<Node<'ts>>,
        tlb: Vec<TopLevelBlock<'ts>>,
    ) -> Self {
        Self {
            node,
            ifnode,
            ifnode_identifier,
            tlb,
        }
    }
}
