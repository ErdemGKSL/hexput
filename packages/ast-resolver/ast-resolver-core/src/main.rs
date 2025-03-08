pub mod ast_structs;
pub mod lexer;
pub mod parser;
pub mod optimizer;
pub mod feature_flags;
pub mod parallel;

use clap::{Arg, Command, ArgAction};
use std::env;
use std::process;
use serde_json::{to_string_pretty, to_string, json};
use feature_flags::FeatureFlags;

struct LocationFilter<'a, T: ?Sized> {
    inner: &'a T,
}

impl<'a, T: serde::Serialize + ?Sized> serde::Serialize for LocationFilter<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde_json::Value;

        let value = serde_json::to_value(&self.inner).map_err(serde::ser::Error::custom)?;
        
        fn filter_locations(value: Value) -> Value {
            match value {
                Value::Object(mut map) => {
                    map.remove("location");
                    
                    let filtered_map = map.into_iter()
                        .map(|(k, v)| (k, filter_locations(v)))
                        .collect();
                    
                    Value::Object(filtered_map)
                },
                Value::Array(arr) => {
                    Value::Array(arr.into_iter().map(filter_locations).collect())
                },
                _ => value,
            }
        }

        let filtered = filter_locations(value);
        filtered.serialize(serializer)
    }
}

fn main() {
    let matches = Command::new("ast-resolver-core")
        .version("0.1.0")
        .about("AST resolver for a custom scripting language")
        .arg(Arg::new("code")
            .help("Code to parse")
            .action(ArgAction::Set))
        .arg(Arg::new("minify")
            .long("minify")
            .help("Minify the output JSON (remove whitespace)")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("no-object-constructions")
            .long("no-object-constructions")
            .help("Disable object literal construction")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("no-array-constructions")
            .long("no-array-constructions")
            .help("Disable array literal construction")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("no-object-navigation")
            .long("no-object-navigation")
            .help("Disable object property access (dot notation and bracket notation)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-variable-declaration")
            .long("no-variable-declaration")
            .help("Disable variable declarations with 'vl'")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-loops")
            .long("no-loops")
            .help("Disable loop statements")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-object-keys")
            .long("no-object-keys")
            .help("Disable keysof operator")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-callbacks")
            .long("no-callbacks")
            .help("Disable callback declarations")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-conditionals")
            .long("no-conditionals")
            .help("Disable if statements")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-return-statements")
            .long("no-return-statements")
            .help("Disable return statements with 'res'")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-loop-control")
            .long("no-loop-control")
            .help("Disable loop control statements (end, continue)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-operators")
            .long("no-operators")
            .help("Disable arithmetic operators (+, *, /)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-equality")
            .long("no-equality")
            .help("Disable equality operator (==)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-assignments")
            .long("no-assignments")
            .help("Disable assignment operator (=)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-source-mapping")
            .long("no-source-mapping")
            .help("Disable source location information in the output JSON")
            .action(ArgAction::SetTrue))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .allow_external_subcommands(true)
        .get_matches();

    let args: Vec<String> = env::args().collect();
    
    let code = extract_code_from_args(&args);
    
    let feature_flags = FeatureFlags::from_cli_args(&matches);
    
    let minify = matches.get_flag("minify");
    
    let include_source_mapping = !matches.get_flag("no-source-mapping");
    
    match process_code(&code, feature_flags) {
        Ok(program) => {

            let json_result = if minify {
                to_string_with_source_mapping(&program, include_source_mapping)
            } else {
                to_string_pretty_with_source_mapping(&program, include_source_mapping)
            };
            
            match json_result {
                Ok(json) => {
                    println!("{}", json);
                },
                Err(e) => {
                    eprintln!("Error serializing AST to JSON: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            let error_json = json!({
                "error": {
                    "type": "ParseError",
                    "message": format!("{}", e)
                }
            });
            
            let error_str = if minify {
                to_string(&error_json)
            } else {
                to_string_pretty(&error_json)
            };
            
            eprintln!("{}", error_str.unwrap());
            process::exit(1);
        }
    }
}

fn extract_code_from_args(args: &[String]) -> String {
    if let Some(pos) = args.iter().position(|arg| arg == "::") {
        if pos + 1 < args.len() {
            args[(pos + 1)..].join(" ")
        } else {
            eprintln!("No code provided after '::'");
            process::exit(1);
        }
    } else if args.len() > 1 {
        let first_non_flag = args.iter()
            .skip(1)
            .position(|arg| !arg.starts_with("--"))
            .map(|pos| pos + 1);
            
        if let Some(pos) = first_non_flag {
            args[pos].clone()
        } else {
            eprintln!("No code provided. Use --help for usage information.");
            process::exit(1);
        }
    } else {
        eprintln!("No code provided. Use --help for usage information.");
        process::exit(1);
    }
}

fn process_code(code: &str, feature_flags: FeatureFlags) -> Result<ast_structs::Program, parser::ParseError> {
    let runtime = parallel::create_runtime();
    
    let tokens = lexer::tokenize(code);
    
    let mut parser = parser::Parser::new(&tokens, feature_flags, code);
    let ast = parser.parse_program()?;
    
    let optimized_ast = optimizer::optimize_ast(ast, &runtime);
    
    Ok(optimized_ast)
}

fn to_string_with_source_mapping(value: &impl serde::Serialize, include_source_mapping: bool) -> Result<String, serde_json::Error> {
    if include_source_mapping {
        to_string(value)
    } else {
        let filtered = LocationFilter {
            inner: value,
        };

        to_string(&filtered)
    }
}

fn to_string_pretty_with_source_mapping(value: &impl serde::Serialize, include_source_mapping: bool) -> Result<String, serde_json::Error> {
    if include_source_mapping {
        to_string_pretty(value)
    } else {
        let filtered = LocationFilter {
            inner: value,
        };

        to_string_pretty(&filtered)
    }
}
