use std::{path::Path, io::{BufReader, BufRead}, fs::File};
use crate::{
    Node,
    Lane,
    Port,
    Dimension,
    set::PortSet,
};

use super::NodePort;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidLaneEntry(u8),
    InvalidNodeEntry(u8),
    InvalidPort(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DynNode(u8);

impl Node for DynNode {
    fn id(&self) -> u8 {
        self.0
    }

    fn from_id(id: u8) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct DynBoardSpec {
    pub nodes: HashMap<String, DynNode>,
    pub node_ports: Vec<NodePort<DynNode>>,
    pub lanes: Vec<Lane>,
    pub bounce_ports: Vec<Port>,
}

fn parse_port(input: &str) -> Result<Port, Error> {
    if let Ok(port) = input.parse() {
        Ok(port)
    } else {
        Err(Error::InvalidPort(input.to_string()))
    }
}

pub fn parse<P: AsRef<Path>>(directory: P) -> Result<DynBoardSpec, Error> {
    let nodes_file = BufReader::new(File::open(directory.as_ref().join("nodes.txt"))?);
    let lanes_file = BufReader::new(File::open(directory.as_ref().join("lanes.txt"))?);
    let bounceports_file = BufReader::new(File::open(directory.as_ref().join("bounceports.txt"))?);

    let mut node_counter = 0;

    let mut nodes = HashMap::new();
    let mut node_ports = vec![];
    let mut lanes = vec![];
    let mut bounce_ports = vec![];

    let mut i = 0;

    for line in nodes_file.lines() {
        i += 1;
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let (node_name, port_spec) = line.split_once(":").ok_or(Error::InvalidNodeEntry(i))?;

        let node = *nodes.entry(node_name.to_string()).or_insert_with(|| {
            let node = DynNode(node_counter);
            node_counter += 1;
            node
        });

        node_ports.push(NodePort(node, parse_port(port_spec)?));
    };

    let mut i = 0;

    for line in lanes_file.lines() {
        i += 1;
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let (a, b) = line.split_once(":").ok_or(Error::InvalidLaneEntry(i))?;
        lanes.push(Lane(parse_port(a)?, parse_port(b)?));
    }

    for line in bounceports_file.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        bounce_ports.push(parse_port(&line)?);
    }

    Ok(DynBoardSpec { nodes, node_ports, lanes, bounce_ports })
}

pub fn sanity_check(board_spec: &DynBoardSpec) {
    let mut problems = vec![];
    let mut used_ports = PortSet::empty();
    for NodePort(node, port) in &board_spec.node_ports {
        if used_ports.contains(*port) {
            problems.push(format!(
                "Port {port:?} used more than once (last use in node mapping {node:?})"
            ));
        }
        used_ports.insert(*port);
    }
    for Lane(a, b) in &board_spec.lanes {
        if used_ports.contains(*a) {
            problems.push(format!(
                "Port {a:?} used more than once (last use in lane with port {b:?})"
            ));
        }
        used_ports.insert(*a);
        if used_ports.contains(*b) {
            problems.push(format!(
                "Port {b:?} used more than once (last use in lane with port {a:?})"
            ));
        }
        used_ports.insert(*b);
    }
    for port in &board_spec.bounce_ports {
        if used_ports.contains(*port) {
            problems.push(format!(
                "Port {port:?} used more than once (last use as bounce port)"
            ));
        }
        used_ports.insert(*port);
    }
    for problem in &problems {
        println!("Found problem: {}", problem);
    }
    let expected = PortSet::full();
    if used_ports != expected {
        println!("Not all ports have been used. Diff:");
        expected.print_diff(&used_ports);

        panic!("Sanity check failed");
    }
    if problems.len() > 0 {
        panic!("All ports were used, but problems have been detected");
    }
}

use quote::quote;
use proc_macro2::{Span, TokenStream};
use syn::{Ident, LitStr, LitByte};

pub fn generate_board_spec_code(board_spec: &DynBoardSpec) -> TokenStream {
    let mut pairs: Vec<_> = board_spec.nodes.iter().collect();
    pairs.sort_by_key(|(_, node)| node.id());

    let mut variants = vec![];
    let mut as_str_arms = vec![];
    let mut from_str_arms = vec![];

    let mut node_tokens = HashMap::new();

    for (name, node) in pairs {
        let variant_name = if name.parse::<u32>().is_ok() {
            format!("_{name}")
        } else {
            name.to_string()
        };
        let id = node.id();
        let variant_token = Ident::new(&variant_name, Span::call_site());
        let name_lit = LitStr::new(&name, Span::call_site());
        let full_variant = quote!(Node::#variant_token);
        variants.push(quote!(#variant_token = #id));
        as_str_arms.push(quote!(#full_variant => #name_lit));
        from_str_arms.push(quote!(#name_lit => Ok(#full_variant)));
        node_tokens.insert(id, full_variant);
    }

    let node_count = board_spec.nodes.len();
    let lane_count = board_spec.lanes.len();
    let bounce_port_count = board_spec.bounce_ports.len();

    let quoted_port = |port: Port| {
        let chip = LitByte::new(port.chip_id().ascii(), Span::call_site());
        let dimension = if port.dimension() == Dimension::X {
            quote!(Dimension::X)
        } else {
            quote!(Dimension::Y)
        };
        let index = port.index();
        quote!(Port::new(ChipId::from_ascii(#chip), #dimension, #index))
    };

    let node_ports = board_spec.node_ports.iter().map(|NodePort(node, port)| {
        let node = node_tokens.get(&node.id());
        let port = quoted_port(*port);
        quote!(NodePort(#node, #port))
    });

    let lanes = board_spec.lanes.iter().map(|Lane(a, b)| {
        let a = quoted_port(*a);
        let b = quoted_port(*b);
        quote!(Lane(#a, #b))
    });

    let bounce_ports = board_spec.bounce_ports.iter().map(|port| quoted_port(*port));

    let node_port_count = board_spec.node_ports.len();

    let node_count_u8 = node_count as u8;

    quote!(
        use jumperless_types::{
            Node as NodeTrait,
            board_spec::NodePort,
            Port,
            ChipId,
            Dimension,
            Lane,
        };

        #[repr(u8)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum Node {
            #(#variants),*
        }

        impl NodeTrait for Node {
            fn id(&self) -> u8 {
                *self as u8
            }

            fn from_id(id: u8) -> Self {
                if id >= #node_count_u8 {
                    panic!("node id out of range");
                }
                unsafe { core::mem::transmute(id) }
            }
        }

        impl Node {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms),*
                }
            }
        }

        #[derive(Debug)]
        pub struct InvalidNode;

        impl core::str::FromStr for Node {
            type Err = InvalidNode;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(InvalidNode),
                }
            }
        }

        pub type BoardSpec = jumperless_types::board_spec::BoardSpec<Node, #node_port_count, #lane_count, #bounce_port_count>;
        pub type Board = jumperless_types::board::Board<Node, #node_port_count, #lane_count, #bounce_port_count>;

        pub fn board_spec() -> BoardSpec  {
            BoardSpec {
                node_ports: [#(#node_ports),*],
                lanes: [#(#lanes),*],
                bounce_ports: [#(#bounce_ports),*],
            }
        }

        pub fn init_board() -> Board {
            Board::new(board_spec())
        }
    )
}
