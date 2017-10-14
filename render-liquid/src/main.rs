extern crate clap;
extern crate liquid;
extern crate toml;

use clap::{App, Arg, ArgMatches};
use liquid::{Renderable, Context};
use std::fs::File;
use std::io;
use std::io::prelude::*;

struct CmdOptions<'a> {
    template_path: &'a str,
    values_path: Option<&'a str>,
    output_path: Option<&'a str>,
}

fn main() {
    let clap_app = App::new("render-liquid")
        .version("0.1.0")
        .arg(Arg::with_name("TEMPLATE")
            .help("Liquid template file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("values")
            .short("t")
            .long("toml")
            .value_name("VALUES.toml")
            .help("Set the TOML file containing values")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("OUTPUT")
            .help("Redirect the output to the file")
            .takes_value(true));

    match parse_and_render(clap_app) {
        Ok(()) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn parse_and_render(clap_app: App) -> Result<(), String> {
    let args = clap_app.get_matches();
    let cmd_opts = parse_options(&args)?;
    let template = parse_template(&cmd_opts.template_path)?;
    let mut values = parse_values(&cmd_opts.values_path)?;
    render(&cmd_opts.output_path, &template, &mut values)
}

fn parse_options<'a>(args: &'a ArgMatches) -> Result<CmdOptions<'a>, String> {
    let template_path = args.value_of("TEMPLATE").ok_or("Can't get template")?;
    let values_path = args.value_of("values");
    let output_path = args.value_of("output");

    let options = CmdOptions{ template_path, values_path, output_path };
    Ok(options)
}

fn parse_template(path: &str) -> Result<liquid::Template, String> {
    liquid::parse_file(path, Default::default())
        .map_err(|e| format!("Can't parse the template at {}. {}", path, e))
}

fn parse_values(path: &Option<&str>) -> Result<Context, String> {
    let mut values_toml = String::new();
    if let &Some(path) = path {
        File::open(path).and_then(|mut f| f.read_to_string(&mut values_toml))
            .map_err(|e| format!("Can't read values from {}. {}", path, e))?;
    } else {
        io::stdin().read_to_string(&mut values_toml)
            .map_err(|e| format!("Can't read values from stdin. {}", e))?;
    }

    let value = values_toml.parse::<toml::Value>()
        .map_err(|e| format!("Can't parse TOML values. {}", e))?;
    
    match value.as_table() {
        Some(table) => {
            let mut context = Context::new();
            for (key, value) in table.iter() {
                match value {
                    &toml::Value::Integer(n) => context.set_val(key, liquid::Value::Num(n as f32)),
                    &toml::Value::Float(f) => context.set_val(key, liquid::Value::Num(f as f32)),
                    &toml::Value::Boolean(b) => context.set_val(key, liquid::Value::Bool(b)),
                    &toml::Value::String(ref s) => context.set_val(key, liquid::Value::Str(s.to_string())),
                    _ => unimplemented!(),
                };
            }
            Ok(context)
        }
        None => Err("Can't parse the top level item it the TOML file as table.".to_string())
    }
}

fn render(path: &Option<&str>, template: &liquid::Template, context: &mut Context) -> Result<(), String> {
    let rendered = template.render(context)
        .map_err(|e| format!("Can't render the template. {}", e))?;
    
    let rendered = rendered.ok_or_else(|| "Nothing to render".to_string())?;

    if let &Some(path) = path {
        match File::create(&path).and_then(|mut f| write!(&mut f, "{}", rendered)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Can't write to {}. {}", path, e)),
        }
    } else {
        print!("{}", rendered);
        // io::stdout().flush().unwrap();
        Ok(())
    }
}