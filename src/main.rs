use anyhow::{bail, Context, Result};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
struct Spec {
    module: String,
    input_name: String,
    input_type: String,
    output_name: String,
    output_type: String,
    domain_min: i64,
    domain_max: i64,
    requires: Vec<String>,
    ensures: Vec<String>,
    objective: Option<(String, String)>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    if command != "compile" {
        bail!("usage: axiom-spec compile <input.ax> --out <output.aix>");
    }

    let input = PathBuf::from(args.next().context("missing input .ax file")?);
    let flag = args.next().context("missing --out")?;
    if flag != "--out" {
        bail!("expected --out, got {flag}");
    }
    let output = PathBuf::from(args.next().context("missing output .aix file")?);

    let source = fs::read_to_string(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let spec = parse_spec(&source)?;
    fs::write(&output, emit_ir(&spec))
        .with_context(|| format!("failed to write {}", output.display()))?;
    println!("compiled {} -> {}", input.display(), output.display());
    Ok(())
}

fn parse_spec(source: &str) -> Result<Spec> {
    let mut module = None;
    let mut input = None;
    let mut output = None;
    let mut domain = None;
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    let mut objective = None;
    let mut saw_header = false;

    for (index, raw) in source.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            ["axiom", "0.1"] => saw_header = true,
            ["module", name] => module = Some((*name).to_owned()),
            ["input", name, ty] => input = Some(((*name).to_owned(), (*ty).to_owned())),
            ["output", name, ty] => output = Some(((*name).to_owned(), (*ty).to_owned())),
            ["domain", name, min, max] => {
                domain = Some((
                    (*name).to_owned(),
                    min.parse::<i64>().context("invalid domain minimum")?,
                    max.parse::<i64>().context("invalid domain maximum")?,
                ));
            }
            ["requires", rest @ ..] if !rest.is_empty() => requires.push(rest.join(" ")),
            ["ensures", rest @ ..] if !rest.is_empty() => ensures.push(rest.join(" ")),
            ["objective", metric, direction] => {
                objective = Some(((*metric).to_owned(), (*direction).to_owned()));
            }
            _ => bail!("line {}: unsupported syntax: {line}", index + 1),
        }
    }

    if !saw_header {
        bail!("missing `axiom 0.1` header");
    }
    let module = module.context("missing module declaration")?;
    let (input_name, input_type) = input.context("missing input declaration")?;
    let (output_name, output_type) = output.context("missing output declaration")?;
    let (domain_name, domain_min, domain_max) = domain.context("missing finite domain")?;
    if domain_name != input_name {
        bail!("domain must currently target the single input `{input_name}`");
    }
    if domain_min > domain_max {
        bail!("domain minimum must not exceed maximum");
    }
    if input_type != "i64" || output_type != "i64" {
        bail!("v0.1 currently supports i64 -> i64 modules only");
    }
    if ensures.is_empty() {
        bail!("at least one ensures clause is required");
    }

    Ok(Spec {
        module,
        input_name,
        input_type,
        output_name,
        output_type,
        domain_min,
        domain_max,
        requires,
        ensures,
        objective,
    })
}

fn canonical_expr(expr: &str) -> String {
    expr.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn emit_ir(spec: &Spec) -> String {
    let mut out = String::new();
    out.push_str("AXIOM-IR/1\n");
    out.push_str(&format!("module={}\n", spec.module));
    out.push_str(&format!("input.name={}\n", spec.input_name));
    out.push_str(&format!("input.type={}\n", spec.input_type));
    out.push_str(&format!("output.name={}\n", spec.output_name));
    out.push_str(&format!("output.type={}\n", spec.output_type));
    out.push_str(&format!("domain.min={}\n", spec.domain_min));
    out.push_str(&format!("domain.max={}\n", spec.domain_max));
    for (i, clause) in spec.requires.iter().enumerate() {
        out.push_str(&format!("requires.{i}={}\n", canonical_expr(clause)));
    }
    for (i, clause) in spec.ensures.iter().enumerate() {
        out.push_str(&format!("ensures.{i}={}\n", canonical_expr(clause)));
    }
    if let Some((metric, direction)) = &spec.objective {
        out.push_str(&format!("objective.metric={metric}\n"));
        out.push_str(&format!("objective.direction={direction}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_abs_spec() {
        let spec = parse_spec("axiom 0.1\nmodule abs\ninput x i64\noutput result i64\ndomain x -2 2\nensures result >= 0\n").unwrap();
        let ir = emit_ir(&spec);
        assert!(ir.contains("module=abs"));
        assert!(ir.contains("domain.min=-2"));
    }
}
