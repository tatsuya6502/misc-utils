extern crate clap;
extern crate liquid;
extern crate toml;

use clap::{App, Arg, ArgMatches};
use liquid::{Renderable, Context};
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub(crate) struct CmdOptions<'a> {
    pub(crate) template_path: &'a str,
    pub(crate) values_path: Option<&'a str>,
    pub(crate) output_path: Option<&'a str>,
}

fn main() {
    let args = App::new("render-liquid")
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
                .takes_value(true))
        .get_matches();

    let result = parse_options(&args).and_then(|cmd_opts| parse_and_render(&cmd_opts));

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(8);
    }
}

pub(crate) fn parse_and_render(cmd_opts: &CmdOptions) -> Result<(), String> {
    let template = parse_template(&cmd_opts.template_path)?;
    let mut values = parse_values(&cmd_opts.values_path)?;
    render(&cmd_opts.output_path, &template, &mut values)
}

fn parse_options<'a>(args: &'a ArgMatches) -> Result<CmdOptions<'a>, String> {
    let template_path = args.value_of("TEMPLATE").ok_or(
        "Can't get template".to_string(),
    )?;
    let values_path = args.value_of("values");
    let output_path = args.value_of("output");

    let options = CmdOptions {
        template_path,
        values_path,
        output_path,
    };
    Ok(options)
}

fn parse_template(path: &str) -> Result<liquid::Template, String> {
    liquid::parse_file(path, Default::default()).map_err(|e| {
        format!("Can't parse the template at {}. {}", path, e)
    })
}

fn parse_values(path: &Option<&str>) -> Result<Context, String> {
    let mut values_toml = String::new();
    if let &Some(path) = path {
        File::open(path)
            .and_then(|mut f| f.read_to_string(&mut values_toml))
            .map_err(|e| format!("Can't read values from {}. {}", path, e))?;
    } else {
        io::stdin().read_to_string(&mut values_toml).map_err(|e| {
            format!("Can't read values from stdin. {}", e)
        })?;
    }

    let value = values_toml.parse::<toml::Value>().map_err(|e| {
        format!("Can't parse TOML values. {}", e)
    })?;

    match value.as_table() {
        Some(table) => {
            let mut context = Context::new();
            for (key, value) in table.iter() {
                context.set_val(key, convert(value));
            }
            Ok(context)
        }
        None => Err(
            "Can't parse the top level item in the TOML file as table.".to_string(),
        ),
    }
}

fn convert(toml: &toml::Value) -> liquid::Value {
    match toml {
        &toml::Value::Integer(i) => liquid::Value::Num(i as f32),
        &toml::Value::Float(f) => liquid::Value::Num(f as f32),
        &toml::Value::Boolean(b) => liquid::Value::Bool(b),
        &toml::Value::String(ref s) => liquid::Value::Str(s.to_string()),
        &toml::Value::Datetime(ref d) => liquid::Value::Str(d.to_string()),
        &toml::Value::Array(ref arr) => liquid::Value::Array(
            arr.into_iter().map(|v| convert(&v)).collect(),
        ),
        &toml::Value::Table(ref table) => liquid::Value::Object(
            table
                .into_iter()
                .map(|(k, v)| (k.to_string(), convert(&v)))
                .collect(),
        ),
    }
}

fn render(
    path: &Option<&str>,
    template: &liquid::Template,
    context: &mut Context,
) -> Result<(), String> {
    let rendered = template.render(context).map_err(|e| {
        format!("Can't render the template. {}", e)
    })?;

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

#[cfg(test)]
mod tests {

    use super::{CmdOptions, parse_and_render};
    use std::{env, fs};
    use std::io::Read;

    #[test]
    fn gen_xml_without_values() {
        let mut out_path = env::temp_dir();
        out_path.push("render-liquid-rendered-a.xml");
        let _ignore = fs::remove_file(&out_path);

        let cmd_opts = CmdOptions {
            template_path: "test-data/template.xml",
            values_path: Some("test-data/empty-values.toml"),
            output_path: Some(out_path.to_str().unwrap()),
        };
        assert_eq!(Ok(()), parse_and_render(&cmd_opts));

        let expected = r#"<?xml version="1.0" encoding="UTF-8" ?>

<values />

"#;

        let mut output = String::new();
        let _size = fs::File::open(&out_path)
            .and_then(|mut f| f.read_to_string(&mut output))
            .expect(&format!("Can't read from output file {:?}", out_path));

        assert_eq!(expected, output);

        let _ignore = fs::remove_file(&out_path);
    }

    #[test]
    fn gen_xml_with_values() {
        let mut out_path = env::temp_dir();
        out_path.push("render-liquid-rendered-b.xml");
        let _ignore = fs::remove_file(&out_path);

        let cmd_opts = CmdOptions {
            template_path: "test-data/template.xml",
            values_path: Some("test-data/values.toml"),
            output_path: Some(out_path.to_str().unwrap()),
        };
        assert_eq!(Ok(()), parse_and_render(&cmd_opts));

        let expected = r#"<?xml version="1.0" encoding="UTF-8" ?>

<values>
  <bool_val>true</bool_val>
  <int_val>123</int_val>
  <float_val>456.7</float_val>
  <string_val>Hello</string_val>
  <array_val>

    <ip_addr>172.17.0.2</ip_addr>

    <ip_addr>172.17.0.3</ip_addr>

  </array_val>
</values>

"#;

        let mut output = String::new();
        let _size = fs::File::open(&out_path)
            .and_then(|mut f| f.read_to_string(&mut output))
            .expect(&format!("Can't read from output file {:?}", out_path));

        assert_eq!(expected, output);

        let _ignore = fs::remove_file(&out_path);
    }

}
