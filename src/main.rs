use anyhow::{bail, Context, Result};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
struct Clause {
    id: String,
    expr: String,
}

#[derive(Debug, Clone)]
enum Domain {
    Unbounded,
    Range { min: i64, max: i64 },
}

#[derive(Debug, Clone)]
struct Spec {
    module: String,
    input_name: String,
    input_type: String,
    output_name: String,
    output_type: String,
    domain: Domain,
    requires: Vec<Clause>,
    ensures: Vec<Clause>,
    objectives: Vec<(String, String)>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("compile") {
        bail!("usage: axiom-spec compile <input.ax> --out <output.aix>");
    }

    let input = PathBuf::from(args.next().context("missing input .ax file")?);
    if args.next().as_deref() != Some("--out") {
        bail!("expected --out");
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

fn parse_clause(rest: &str, prefix: &str, index: usize) -> Clause {
    let trimmed = rest.trim();
    if let Some((candidate, expr)) = trimmed.split_once(':') {
        let id = candidate.trim();
        if !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Clause {
                id: id.to_owned(),
                expr: canonical_expr(expr),
            };
        }
    }

    Clause {
        id: format!("{prefix}{index}"),
        expr: canonical_expr(trimmed),
    }
}

fn parse_spec(source: &str) -> Result<Spec> {
    let mut saw_header = false;
    let mut module = None;
    let mut input = None;
    let mut output = None;
    let mut domain = None;
    let mut requires = Vec::new();
    let mut ensures = Vec::new();
    let mut objectives = Vec::new();

    for (index, raw) in source.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line == "axiom 0.2" {
            saw_header = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("module ") {
            module = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("input ") {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() != 2 {
                bail!("line {}: input expects <name> <type>", index + 1);
            }
            input = Some((words[0].to_owned(), words[1].to_owned()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("output ") {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() != 2 {
                bail!("line {}: output expects <name> <type>", index + 1);
            }
            output = Some((words[0].to_owned(), words[1].to_owned()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("domain ") {
            let words: Vec<_> = rest.split_whitespace().collect();
            domain = match words.as_slice() {
                [_name, "unbounded"] => Some(Domain::Unbounded),
                [_name, min, max] => {
                    let min = min.parse::<i64>().context("invalid domain minimum")?;
                    let max = max.parse::<i64>().context("invalid domain maximum")?;
                    if min > max {
                        bail!("line {}: domain minimum exceeds maximum", index + 1);
                    }
                    Some(Domain::Range { min, max })
                }
                _ => bail!(
                    "line {}: domain expects <name> unbounded or <name> <min> <max>",
                    index + 1
                ),
            };
            continue;
        }
        if let Some(rest) = line.strip_prefix("requires ") {
            requires.push(parse_clause(rest, "requires-", requires.len()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("ensures ") {
            ensures.push(parse_clause(rest, "ensures-", ensures.len()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("objective ") {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() != 2 || !matches!(words[1], "min" | "max") {
                bail!("line {}: objective expects <metric> <min|max>", index + 1);
            }
            objectives.push((words[0].to_owned(), words[1].to_owned()));
            continue;
        }

        bail!("line {}: unsupported syntax: {line}", index + 1);
    }

    if !saw_header {
        bail!("missing `axiom 0.2` header");
    }
    let module = module.context("missing module declaration")?;
    let (input_name, input_type) = input.context("missing input declaration")?;
    let (output_name, output_type) = output.context("missing output declaration")?;
    let domain = domain.context("missing domain declaration")?;

    if input_type != "int" || output_type != "int" {
        bail!("v0.2 symbolic core currently supports int -> int modules");
    }
    if ensures.is_empty() {
        bail!("at least one ensures clause is required");
    }
    if requires.is_empty() {
        requires.push(Clause {
            id: "requires-0".to_owned(),
            expr: "true".to_owned(),
        });
    }

    Ok(Spec {
        module,
        input_name,
        input_type,
        output_name,
        output_type,
        domain,
        requires,
        ensures,
        objectives,
    })
}

fn canonical_expr(expr: &str) -> String {
    expr.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn emit_ir(spec: &Spec) -> String {
    let mut out = String::from("AXIOM-IR/2\n");
    out.push_str(&format!("module={}\n", spec.module));
    out.push_str(&format!("input.0.name={}\n", spec.input_name));
    out.push_str(&format!("input.0.type={}\n", spec.input_type));
    out.push_str(&format!("output.name={}\n", spec.output_name));
    out.push_str(&format!("output.type={}\n", spec.output_type));

    match spec.domain {
        Domain::Unbounded => {
            out.push_str(&format!("domain.{}.kind=unbounded\n", spec.input_name));
        }
        Domain::Range { min, max } => {
            out.push_str(&format!("domain.{}.kind=range\n", spec.input_name));
            out.push_str(&format!("domain.{}.min={min}\n", spec.input_name));
            out.push_str(&format!("domain.{}.max={max}\n", spec.input_name));
        }
    }

    for (index, clause) in spec.requires.iter().enumerate() {
        out.push_str(&format!("requires.{index}.id={}\n", clause.id));
        out.push_str(&format!("requires.{index}.expr={}\n", clause.expr));
    }
    for (index, clause) in spec.ensures.iter().enumerate() {
        out.push_str(&format!("ensures.{index}.id={}\n", clause.id));
        out.push_str(&format!("ensures.{index}.expr={}\n", clause.expr));
    }
    for (index, (metric, direction)) in spec.objectives.iter().enumerate() {
        out.push_str(&format!("objective.{index}.metric={metric}\n"));
        out.push_str(&format!("objective.{index}.direction={direction}\n"));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_unbounded_abs() {
        let spec = parse_spec(
            "axiom 0.2\nmodule abs\ninput x int\noutput result int\ndomain x unbounded\nensures nonnegative: result >= 0\n",
        )
        .unwrap();
        let ir = emit_ir(&spec);
        assert!(ir.contains("AXIOM-IR/2"));
        assert!(ir.contains("domain.x.kind=unbounded"));
        assert!(ir.contains("ensures.0.id=nonnegative"));
    }
}
