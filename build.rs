use std::{
    env,
    fs::{read_dir, read_to_string, File},
    io::Write,
    path::PathBuf,
};

use regex::Regex;

const COMMAND_REGEX: &str = r"pub fn (.*)\(app: &mut Application\) -> Result<\(\)>";

fn main() {
    generate_handler();
}

fn generate_handler() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file_pathbuf = PathBuf::new().join(out_dir).join("handle_map");

    let mut out_file = File::create(out_file_pathbuf).unwrap();
    out_file
        .write(
            r"{
    let mut handles: HashMap<&'static str, fn(&mut Application) -> Result<()>> = HashMap::new();
"
            .as_bytes(),
        )
        .unwrap();

    let expression = Regex::new(COMMAND_REGEX).expect("Failed to compile command matching regex");
    let readdir = read_dir("./src/application/handler/").unwrap();

    for entry in readdir {
        if let Ok(entry) = entry {
            let path = entry.path();
            let module_name = entry
                .file_name()
                .into_string()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .to_owned();

            let content = read_to_string(path).unwrap();
            for captures in expression.captures_iter(&content) {
                let function_name = captures.get(1).unwrap().as_str();
                write(&mut out_file, &module_name, function_name);
            }
        }
    }

    out_file
        .write(
            r"
    handles
}"
            .as_bytes(),
        )
        .unwrap();
}

fn write(output: &mut File, module_name: &str, function_name: &str) {
    output
        .write(
            format!(
                "    handles.insert(\"{}::{}\", {}::{});\n",
                module_name, function_name, module_name, function_name
            )
            .as_bytes(),
        )
        .unwrap();
}
