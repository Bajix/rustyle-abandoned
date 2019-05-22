use crate::CodeGenerator;
use metadata::RootMetadataProcessor;
use node::{DeclarationNode, MetadataNode, RulesetNode, RulesetType};
use runtime::global::ROOT_METADATA_PROCESSORS;
use runtime_facade::CompileContext;
use std::collections::HashMap;
use std::slice::SliceConcatExt;

impl CodeGenerator for RulesetNode {
    /// Generate code from [`RulesetNode`].
    /// 
    /// [`DeclarationNode`]: node::RulesetNode
    fn generate_code(&self, base_class: &str, context: &mut CompileContext) -> String {
        if self.ruleset_type == RulesetType::Root {
            let root_metadata_processors = ROOT_METADATA_PROCESSORS.lock().unwrap();

            let mut processors =
                HashMap::<String, (&Box<dyn RootMetadataProcessor>, Vec<MetadataNode>)>::new();

            for processor in root_metadata_processors.values() {
                processors.insert(processor.name().to_string(), (processor, Vec::new()));
            }

            for metadata in self.metadatas.clone() {
                if !processors.contains_key(&metadata.method_name) {
                    metadata.range.error("Unknown metadata").emit();
                    continue;
                }

                processors
                    .get_mut(&metadata.method_name.clone())
                    .expect("Guaranteed by before if")
                    .1
                    .push(metadata);
            }

            for (processor, metadatas) in processors.values() {
                (*processor).process(context, metadatas.to_vec());
            }
        }

        let mut result = String::new();

        let base = match &self.ruleset_type {
            RulesetType::Root => format!(".{}", base_class),
            RulesetType::Selector(group) => format!(
                "\n{}",
                group
                    .iter()
                    .map(|selector| selector.stringify(base_class.to_string()))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        };

        result.push_str(&base);

        result.push_str(" {\n");

        let mut appeared_nodes = HashMap::<String, (&DeclarationNode, bool)>::new();

        let alert_duplicated = |node: &DeclarationNode| {
            node.range
                .warning(format!(
                    "Consider removing duplicated property {}",
                    node.name
                ))
                .emit();
        };

        for declaration in &self.declarations {
            let is_duplicated = if let Some(before) = appeared_nodes.get(&declaration.name) {
                alert_duplicated(before.0);
                true
            } else {
                false
            };

            appeared_nodes.insert(declaration.name.clone(), (declaration, is_duplicated));
        }

        for (node, is_duplicated) in appeared_nodes.values() {
            if *is_duplicated {
                alert_duplicated(node);
            }
            result.push_str("  ");
            result.push_str(&node.generate_code(base_class, context));
            result.push_str("\n");
        }

        result.push_str("}");

        for ruleset in &self.nested_rulesets {
            result.push_str(&ruleset.generate_code(base_class, context));
        }

        result
    }
}
