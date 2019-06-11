extern crate serde;
extern crate serde_yaml;

use std::process;
use std::io;
use std::io::ErrorKind;
use std::fs::{read_to_string};
use serde_yaml::Value;

static DEFAULT_CHISEL_CONF: &'static str = "chisel.yml";
static ERROR_YAML_CONF: &'static str = "ERROR: error reading yaml configuration";

#[derive(Debug)]
struct SnipParams {
    input: String,
    output: String
}

fn execute_command(command: &str, args: Vec<&str>) -> Result<process::Output, std::io::Error>{
    process::Command::new(command).args(args).output()
}

fn build_module() -> process::Output {
    println!("\nBuilding ewasm module...");

    let args = vec!["build",
                    "--target=wasm32-unknown-unknown",
                    "--release"];

    let output = execute_command("cargo", args).unwrap();
    
    return output;
}

fn execute_chisel() -> Result<String, io::Error> {
    println!("\nExecuting chisel...");

    match execute_command("chisel", vec!["run"]) {
        Ok(output) => {
            let result = String::from_utf8(output.stdout).expect("Not UTF-8");
            println!("{}", result);
            return Ok(result);
        },
        Err(e) => {
            //println!("{:?}", e);
            if e.kind() == ErrorKind::NotFound {
                let error = "ERROR: chisel not found, please install chisel.";
                println!("{}", error);
            }
            return Err(e);
        }
    }
}

fn get_snip_params() -> Result<SnipParams, &'static str> {
    let mut params = SnipParams {
        input: String::from(""),
        output: String::from("")
    };

    match read_to_string(DEFAULT_CHISEL_CONF) {
        Ok(conf) => {
            if let Ok(ruleset) = serde_yaml::from_str::<Value>(conf.as_str()) {
                if let Value::Mapping(rules) = ruleset {
                    for (name, mut config) in rules.iter().filter(|(left, right)| match (left, right) {
                        (Value::String(_s), Value::Mapping(_m)) => true,
                        _ => false,
                    }) {
                        let input = config.get(&Value::String(String::from("file"))).unwrap().as_str().unwrap();
                        let output = config.get(&Value::String(String::from("output"))).unwrap().as_str().unwrap();

                        params.input = input.to_string();
                        params.output = output.to_string();

                        return Ok(params);
                    };
                }
            }
            
        },
        Err(_) => {
            return Err(ERROR_YAML_CONF);
        }
    }
    Err(ERROR_YAML_CONF)
}

fn execute_wasm_snip(params: SnipParams) {
    print!("\nMinifying bytecode (wasm-snip)... ");
    let args = vec!["--snip-rust-panicking-code", params.input.as_str(), "-o", params.output.as_str()];
    match execute_command("wasm-snip", args) {
        Ok(output) => {
            if output.status.success() {
                println!("OK\n{}", String::from_utf8(output.stdout).expect("Not UTF-8"));

            } else {
                println!("{}", String::from_utf8(output.stderr).expect("Not UTF-8"));
            }

        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn main() {
    let output = build_module();
    
    if !output.status.success() {
        println!("{}", String::from_utf8(output.stderr).expect("Not UTF-8"));
        println!("ERROR: error building wasm module, confirm target `wasm32-unknown-unknown` is installed");
        return;
    }    

    if execute_chisel().is_err() {
        return;
    }

    let snip_params = get_snip_params().unwrap();

    execute_wasm_snip(snip_params);

    println!("Finished");

    //if chisel_result.
}

