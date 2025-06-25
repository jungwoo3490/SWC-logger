use std::path::Path;
use swc_core::ecma::{
    ast::*,
    visit::{as_folder, FoldWith, VisitMut},
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
        Ok(_transformer) => {
            println!("Config parsing successful from: {}", config_path);
            program
        }
        Err(e) => {
            eprintln!("Config parsing error: {}", e);
            program
        }
    }
}
