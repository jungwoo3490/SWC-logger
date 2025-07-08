use std::path::Path;
use swc_core::ecma::{
    ast::*,
    visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod config;
use config::{parse_config, find_config_file, LoggerConfig};

pub struct LoggerTransformer {
    config: LoggerConfig,
}

impl LoggerTransformer {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = parse_config(Path::new(config_path))?;
        Ok(Self { config })
    }
}

impl VisitMut for LoggerTransformer {
    fn visit_mut_jsx_element(&mut self, jsx_element: &mut JSXElement) {
 
        if let JSXElementName::Ident(ident) = &jsx_element.opening.name {
            println!("Found JSX element: {}", ident.sym);
            
            for attr in &jsx_element.opening.attrs {
                match attr {
                    JSXAttrOrSpread::JSXAttr(jsx_attr) => {
                        if let JSXAttrName::Ident(attr_name) = &jsx_attr.name {
                            println!("Attribute: {}", attr_name.sym);
                            
                            if attr_name.sym.starts_with("data-") {
                                println!("Found data attribute: {}", attr_name.sym);
                                
                                let data_key = attr_name.sym.as_str();
                                if let Some(rule) = self.config.rules.get(data_key) {
                                    println!("Found matching rule for {}: type={}", data_key, rule.r#type);
                                }
                            }
                        }
                    }
                    JSXAttrOrSpread::SpreadElement(_) => {
                        println!("Spread attribute found");
                    }
                }
            }
        }
        
        jsx_element.visit_mut_children_with(self);
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {

    let config_path = match find_config_file() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Config file not found: {}", e);
            return program;
        }
    };
    
    match LoggerTransformer::new(&config_path) {
        Ok(transformer) => {
            println!("Config parsing successful from: {}", config_path);
            program.fold_with(&mut as_folder(transformer))
        }
        Err(e) => {
            eprintln!("Config parsing error: {}", e);
            program
        }
    }
}
