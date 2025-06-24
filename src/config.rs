use std::process::Command;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggerRule {
    pub task: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggerConfig {
    pub rules: std::collections::HashMap<String, LoggerRule>,
    pub options: Option<ConfigOptions>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigOptions {
    pub enabled: Option<bool>,
    pub debug_mode: Option<bool>,
    pub batch_size: Option<u32>,
}

pub fn parse_config(config_path: &Path) -> Result<LoggerConfig, Box<dyn std::error::Error>> {
    let is_typescript = config_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "ts")
        .unwrap_or(false);
    
    let output = if is_typescript {
        Command::new("npx")
            .arg("tsx")
            .arg("-e")
            .arg(format!(
                r#"
                import config from '{}';
                
                const serialized = JSON.stringify(config, (key, value) => {{
                    if (typeof value === 'function') {{
                        return value.toString();
                    }}
                    return value;
                }});
                
                console.log(serialized);
                "#,
                config_path.display()
            ))
            .output()?
    } else {
        Command::new("node")
            .arg("-e")
            .arg(format!(
                r#"
                const path = require('path');
                const configPath = '{}';
                
                (async () => {{
                    let config;
                    try {{
                        const module = await import(path.resolve(configPath));
                        config = module.default || module;
                    }} catch (e) {{
                        delete require.cache[require.resolve(path.resolve(configPath))];
                        config = require(path.resolve(configPath));
                    }}
                    
                    const serialized = JSON.stringify(config, (key, value) => {{
                        if (typeof value === 'function') {{
                            return value.toString();
                        }}
                        return value;
                    }});
                    
                    console.log(serialized);
                }})().catch(console.error);
                "#,
                config_path.display()
            ))
            .output()?
    };

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to parse config: {}", error).into());
    }

    let json_str = String::from_utf8(output.stdout)?;
    let config: LoggerConfig = serde_json::from_str(&json_str.trim())?;
    
    Ok(config)
}

pub fn find_config_file() -> Result<String, Box<dyn std::error::Error>> {
    let possible_configs = [
        "logger.config.js",
        "logger.config.ts", 
    ];
    
    for config_path in possible_configs {
        if fs::metadata(config_path).is_ok() {
            println!("Found config file: {}", config_path);
            return Ok(config_path.to_string());
        }
    }
    
    Err("No logger config file found in project root. Supported files: logger.config.js, logger.config.ts".into())
}