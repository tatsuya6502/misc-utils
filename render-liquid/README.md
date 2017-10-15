# Render Liquid [![CircleCI](https://circleci.com/gh/tatsuya6502/tiny-utils/tree/master.svg?style=svg)](https://circleci.com/gh/tatsuya6502/tiny-utils/tree/master)

`render-liquid` command renders
a [Liquid](https://shopify.github.io/liquid/) template with supplied
values in [TOML](https://github.com/toml-lang/toml) format. The
motivation of creating this command was to make it easy to generate
XML based configuration files from a shell script.


## Usage

```console
# Use the Docker image containing the command. (It is only ~X MB)
# This should run on Docker on Linux, macOS and Windows with Docker installed.
$ alias render-liquid='docker run -i --rm -v $(pwd):/files quay.io/tatsuya6502/render-liquid'

# Create a template in Liquid.
$ cat <<'EOF' > template.txt
{% if user_name %}
  Hey {{ user_name }}!
{% else %}
  Hello, world!
{% endif %}
EOF

$ echo '' | render-liquid /files/template.txt

  Hello, world!

$ echo 'user_name = "Tatsuya"' | render-liquid /files/template.txt

  Hey Tatsuya!

```

```console
$ render-liquid --help
render-liquid 0.1.0

USAGE:
    rendar-liquid [OPTIONS] <TEMPLATE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <OUTPUT>       Redirect the output to the file
    -t, --toml <VALUES.toml>    Set the TOML file containing values

ARGS:
    <TEMPLATE>    Liquid template file
```


## Supported Data Types

This command only supports the following data types in TOML.

- String
- Integer
- Float
- Boolean

I have no plan to extend, but it will be easy to do so because other
data types such as array are already supported in the crates
(libraries) that this command depends on. Pull request will be always
welcome.


## Building the Docker Image

You need Linux, macOS, or Windows with Docker installed.

```console
$ docker build -t render-liquid .
```

The Dockerfile can be found [here](./Dockerfile).

NOTE: Require Rust version: 1.18.0 or newer.


## Dependencies

This command is written in
[Rust programming language](https://www.rust-lang.org) and depends on
the following Rust crates:

- [liquid (liquid-rust)](https://crates.io/crates/liquid) -- Liquid
  templating for Rust
- [toml (toml-rs)](https://crates.io/crates/toml) -- A TOML decoder
  and encoder for Rust, currently compliant with the v0.4.0 version of
  TOML
- [clap](https://crates.io/crates/clap) -- A simple to use, efficient,
  and full featured Command Line Argument Parser
