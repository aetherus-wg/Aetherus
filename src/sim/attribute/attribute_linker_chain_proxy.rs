use arctk_attr::file;
use crate::{ord::Set, sim::AttributeLinkerLinkerLinkerLinkerLinkerLinker};

type LinkerChainStart = AttributeLinkerLinkerLinkerLinkerLinkerLinker;

/// This is the jumping on point for the attribute chain. 
/// This enum serves a dual purpose. It simultaneously acts as as a convenient
/// wrapper around the attribute chain, but it also allows us to neatly resolve
/// a vec of attribute declarations (wrapped by the `LinkerChainEndpoint` type)
/// into an `AttributeChain` variant of the struct. Small, but improved ergonomics
/// for the input files. 
#[file]
#[serde(untagged)]
pub enum AttributeLinkerChainProxy {
    AttributeChain(Vec<LinkerChainStart>),
    Attribute(LinkerChainStart),
}

impl AttributeLinkerChainProxy {
    pub fn resolve(&self) -> LinkerChainStart {
        match self {
            AttributeLinkerChainProxy::Attribute(attr) => attr.clone(),
            AttributeLinkerChainProxy::AttributeChain(attrs) => LinkerChainStart::AttributeChain(attrs.clone())
        }
    }
}

/// Resolves a set of `AttributeLinkerChainProxy` into a set of attribute linker chain endpoints. 
/// This should be used in the `load` implementation in the `ParameterBuilderLoader` struct. 
pub fn attribute_chain_resolve_set(inset: Set<AttributeLinkerChainProxy>) -> Set<LinkerChainStart> {
    let output_pairs = inset.into_iter().map(|(name, proxy)| {
        let attr_linker = proxy.resolve();
        (name, attr_linker)
    })
    .collect();

    Set::from_pairs(output_pairs)
        .expect("Unable to resolve attribute linkers from proxies. ")
}