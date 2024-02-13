// trying shell script generation by upon

use std::{error::Error, process::Command, fmt::Write as _, collections::HashMap};
use once_cell::sync::Lazy;

static TEMPLATE_ENGINE: Lazy<upon::Engine> = Lazy::new(|| {
    let mut engine = upon::Engine::new();
    engine.set_default_formatter(&escape_shell);
    engine.add_template("scalar_value", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/scalar_value.zsh"))).unwrap();
    engine.add_template("array_value", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/array_value.zsh"))).unwrap();
    engine.add_template("associative_array_value", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/associative_array_value.zsh"))).unwrap();
    engine
});


fn escape_shell(formatter: &mut upon::fmt::Formatter<'_>, value: &upon::Value) -> upon::fmt::Result {
    match value {
        upon::Value::None => return Err("Value::None is not supported in shell script template".into()),
        upon::Value::String(s) => formatter.write_str(&shell_escape::escape(s.into()))?,
        upon::Value::Bool(b) => return Err(format!("Value::Bool({}) is not supported in shell script template, because what boolean is depends on syntaxt context", b).into()),
        upon::Value::Integer(i) => formatter.write_str(&i.to_string())?,
        upon::Value::Float(f) => formatter.write_str(&f.to_string())?,
        upon::Value::List(l) => {
            formatter.write_str("(")?;
            for v in l.iter() {
                if let upon::Value::List(_) | upon::Value::Map(_) = v {
                    return Err("nested list or map is not supported in shell script template".into());
                }
                escape_shell(formatter, v)?;
                formatter.write_str(" ")?;
            }
            formatter.write_str(")")?;
        },
        upon::Value::Map(m) => {
            formatter.write_str("(")?;
            for (k, v) in m.iter() {
                if let upon::Value::List(_) | upon::Value::Map(_) = v {
                    return Err("nested list or map is not supported in shell script template".into());
                }
                formatter.write_str(&shell_escape::escape(k.into()))?;
                formatter.write_str(" ")?;
                escape_shell(formatter, v)?;
                formatter.write_str(" ")?;
            }
            formatter.write_str(")")?;
        },
    };
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    check_scalar_input("hello")?;
    check_scalar_input("hello world")?;
    check_scalar_input("''''")?;
    check_scalar_input(1)?;
    check_scalar_input(1.0)?;
    check_scalar_input(1.3)?;
    check_scalar_input(1.3e100)?;

    check_array_input::<&str>(vec![])?;
    check_array_input(vec!["hello", "world"])?;
    check_array_input(vec!["hello world", "''''", "\"\"\"", "\n\n", "(", ")"])?;
    check_array_input(vec![1, 2, 3])?;

    check_associative_array_input::<&str, &str>(HashMap::new())?;
    check_associative_array_input(HashMap::from([("hello", "world"), ("''''", "\"\"\"")]))?;
    check_associative_array_input(HashMap::from([(1, 5.0), (3, 3e23)]))?;

    println!("everything is ok!!");
    Ok(())
}

fn check_scalar_input(value: impl serde::Serialize + std::fmt::Display) -> Result<(), Box<dyn Error>> {
    let serialized_input = value.to_string();
    let script = TEMPLATE_ENGINE.template("scalar_value").render(upon::value!{ a: value }).to_string()?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&script)
        .output()?
        .stdout;

    println!("---- script ----");
    println!("{}", script);
    println!("---- output ----");
    println!("{}", String::from_utf8_lossy(&output));
    println!("---- compare with input ----");
    println!("{}", serialized_input);
    println!("================");
    assert_eq!(String::from_utf8_lossy(&output), String::from(serialized_input) + "\n");
    Ok(())
}

fn check_array_input<V>(value: Vec<V>) -> Result<(), Box<dyn Error>>
where V: serde::Serialize + std::fmt::Display
{
    let serialized_input = value.iter().map(|e| e.to_string() + "\n").collect::<String>();
    let script = TEMPLATE_ENGINE.template("array_value").render(upon::value!{ a: value }).to_string()?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&script)
        .output()?
        .stdout;

    println!("---- script ----");
    println!("{}", script);
    println!("---- output ----");
    println!("{}", String::from_utf8_lossy(&output));
    println!("---- compare with input ----");
    println!("{}", serialized_input);
    println!("================");
    assert_eq!(String::from_utf8_lossy(&output), serialized_input);
    Ok(())
}

fn check_associative_array_input<K, V>(value: HashMap<K, V>) -> Result<(), Box<dyn Error>>
where K: serde::Serialize + std::fmt::Display,
      V: serde::Serialize + std::fmt::Display
{
    let serialized_input = value.iter().map(|(k, v)| k.to_string() + "\n" + &v.to_string() + "\n").collect::<String>();
    let script = TEMPLATE_ENGINE.template("associative_array_value").render(upon::value!{ a: value }).to_string()?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&script)
        .output()?
        .stdout;

    println!("---- script ----");
    println!("{}", script);
    println!("---- output ----");
    println!("{}", String::from_utf8_lossy(&output));
    println!("---- compare with input ----");
    println!("{}", serialized_input);
    println!("================");
    // zsh associative array is not ordered
    // so we can't compare the output with serialized_input directly
    // assert_eq!(String::from_utf8_lossy(&output), serialized_input);
    Ok(())
}

