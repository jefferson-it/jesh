use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Record>,
}

impl Table {
    pub fn new() -> Self {
        Table { columns: Vec::new(), rows: Vec::new() }
    }

    pub fn from_csv(input: &str) -> Result<Table, String> {
        let mut lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Err("empty input".to_string());
        }
        let header = lines.remove(0);
        let cols: Vec<String> = Self::parse_csv_line(header).into_iter()
            .map(|s| s.trim().to_string())
            .collect();
        if cols.is_empty() {
            return Err("empty header".to_string());
        }
        let mut table = Table { columns: cols.clone(), rows: Vec::new() };
        for line in lines {
            let line = line.trim();
            if line.is_empty() { continue; }
            let vals = Self::parse_csv_line(line);
            let mut record = Record { fields: HashMap::new() };
            for (i, col) in cols.iter().enumerate() {
                let val = vals.get(i).map(|s| s.trim()).unwrap_or("");
                record.fields.insert(col.clone(), Self::parse_value(val));
            }
            table.rows.push(record);
        }
        Ok(table)
    }

    pub fn from_tsv(input: &str) -> Result<Table, String> {
        let mut lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Err("empty input".to_string());
        }
        let header = lines.remove(0);
        let cols: Vec<String> = header.split('\t').map(|s| s.trim().to_string()).collect();
        if cols.is_empty() {
            return Err("empty header".to_string());
        }
        let mut table = Table { columns: cols.clone(), rows: Vec::new() };
        for line in lines {
            let line = line.trim();
            if line.is_empty() { continue; }
            let vals: Vec<&str> = line.split('\t').collect();
            let mut record = Record { fields: HashMap::new() };
            for (i, col) in cols.iter().enumerate() {
                let val = vals.get(i).map(|s| s.trim()).unwrap_or("");
                record.fields.insert(col.clone(), Self::parse_value(val));
            }
            table.rows.push(record);
        }
        Ok(table)
    }

    pub fn from_json(input: &str) -> Result<Table, String> {
        let input = input.trim();
        let input = if input.starts_with('[') { input } else { return Err("expected JSON array".to_string()) };
        let parsed: Vec<serde_json::Value> = serde_json::from_str(input)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        let mut columns: Vec<String> = Vec::new();
        let mut rows = Vec::new();
        for obj in &parsed {
            match obj {
                serde_json::Value::Object(map) => {
                    for key in map.keys() {
                        if !columns.contains(key) {
                            columns.push(key.clone());
                        }
                    }
                }
                _ => return Err("expected array of objects".to_string()),
            }
        }
        for obj in &parsed {
            if let serde_json::Value::Object(map) = obj {
                let mut record = Record { fields: HashMap::new() };
                for col in &columns {
                    let val = map.get(col).map(Self::json_to_value).unwrap_or(Value::Null);
                    record.fields.insert(col.clone(), val);
                }
                rows.push(record);
            }
        }
        Ok(Table { columns, rows })
    }

    fn json_to_value(v: &serde_json::Value) -> Value {
        match v {
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Number(n) => {
                n.as_i64().map(Value::Int)
                    .or_else(|| n.as_f64().map(Value::Float))
                    .unwrap_or(Value::Null)
            }
            serde_json::Value::Bool(b) => Value::Bool(*b),
            _ => Value::Null,
        }
    }

    pub fn to_json_string(&self) -> String {
        let mut out = String::from('[');
        for (i, row) in self.rows.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push('{');
            for (j, col) in self.columns.iter().enumerate() {
                if j > 0 { out.push(','); }
                out.push('"');
                out.push_str(&col.escape_default().to_string());
                out.push_str("\":");
                let val = row.fields.get(col).unwrap_or(&Value::Null);
                match val {
                    Value::String(s) => {
                        out.push('"');
                        out.push_str(&s.escape_default().to_string());
                        out.push('"');
                    }
                    Value::Int(n) => out.push_str(&n.to_string()),
                    Value::Float(n) => out.push_str(&n.to_string()),
                    Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
                    Value::Null => out.push_str("null"),
                }
            }
            out.push('}');
        }
        out.push(']');
        out
    }

    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push_str(&Self::csv_escape(col));
        }
        out.push('\n');
        for row in &self.rows {
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 { out.push(','); }
                let val = row.fields.get(col).unwrap_or(&Value::Null);
                out.push_str(&Self::csv_escape(&val.to_string()));
            }
            out.push('\n');
        }
        out
    }

    pub fn to_table(&self) -> String {
        if self.columns.is_empty() {
            return String::new();
        }
        let col_widths: Vec<usize> = self.columns.iter().map(|c| c.len()).collect();
        let mut widths = col_widths.clone();
        for row in &self.rows {
            for (i, col) in self.columns.iter().enumerate() {
                let val = row.fields.get(col).unwrap_or(&Value::Null);
                let w = val.to_string().len();
                if w > widths[i] { widths[i] = w; }
            }
        }
        let total_w: usize = widths.iter().sum::<usize>() + widths.len() * 3 + 1;
        let mut out = String::new();
        out.push('+');
        for w in &widths { out.push_str(&"-".repeat(w + 2)); out.push('+'); }
        out.push('\n');
        out.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            out.push(' ');
            out.push_str(&format!("{:width$}", col, width = widths[i]));
            out.push_str(" |");
        }
        out.push('\n');
        out.push('+');
        for w in &widths { out.push_str(&"-".repeat(w + 2)); out.push('+'); }
        out.push('\n');
        for row in &self.rows {
            out.push('|');
            for (i, col) in self.columns.iter().enumerate() {
                let val = row.fields.get(col).unwrap_or(&Value::Null);
                out.push(' ');
                out.push_str(&format!("{:width$}", val.to_string(), width = widths[i]));
                out.push_str(" |");
            }
            out.push('\n');
        }
        out.push('+');
        for w in &widths { out.push_str(&"-".repeat(w + 2)); out.push('+'); }
        out.push('\n');
        out
    }

    pub fn select(&self, columns: &[String]) -> Table {
        let cols: Vec<String> = columns.iter()
            .filter(|c| self.columns.contains(c))
            .cloned()
            .collect();
        let rows: Vec<Record> = self.rows.iter().map(|r| {
            let mut fields = HashMap::new();
            for c in &cols {
                if let Some(v) = r.fields.get(c) {
                    fields.insert(c.clone(), v.clone());
                }
            }
            Record { fields }
        }).collect();
        Table { columns: cols, rows }
    }

    pub fn where_filter(&self, expr: &str) -> Result<Table, String> {
        let rows: Vec<Record> = self.rows.iter()
            .filter(|r| Self::eval_expr(r, expr).unwrap_or(false))
            .cloned()
            .collect();
        Ok(Table { columns: self.columns.clone(), rows })
    }

    pub fn sort_by_col(&self, col: &str, desc: bool) -> Table {
        let col_idx = self.columns.iter().position(|c| c == col);
        let mut rows = self.rows.clone();
        if let Some(ci) = col_idx {
            rows.sort_by(|a, b| {
                let va = a.fields.get(col).unwrap_or(&Value::Null);
                let vb = b.fields.get(col).unwrap_or(&Value::Null);
                let cmp = Self::cmp_values(va, vb);
                if desc { cmp.reverse() } else { cmp }
            });
        }
        Table { columns: self.columns.clone(), rows }
    }

    fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Int(x), Value::Float(y)) => (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Float(x), Value::Int(y)) => x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(x), Value::String(y)) => x.cmp(y),
            (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn eval_expr(record: &Record, expr: &str) -> Result<bool, String> {
        let expr = expr.trim();
        if expr.is_empty() { return Ok(true); }

        let mut parts: Vec<&str> = Vec::new();
        let mut depth = 0;
        let mut current = String::new();
        let mut in_quote = false;
        for c in expr.chars() {
            match c {
                '"' => in_quote = !in_quote,
                '(' if !in_quote => depth += 1,
                ')' if !in_quote => depth -= 1,
                _ => {}
            }
            if !in_quote && depth == 0 && expr[current.len()..].starts_with(" and ") && current.len() > 0 {
                let before = &expr[..current.len()];
                let after = &expr[current.len() + 5..];
                let left = Self::eval_expr(record, before)?;
                let right = Self::eval_expr(record, after)?;
                return Ok(left && right);
            }
            if !in_quote && depth == 0 && expr[current.len()..].starts_with(" or ") && current.len() > 0 {
                let before = &expr[..current.len()];
                let after = &expr[current.len() + 4..];
                let left = Self::eval_expr(record, before)?;
                let right = Self::eval_expr(record, after)?;
                return Ok(left || right);
            }
            current.push(c);
        }

        let expr = expr.trim();
        if expr.is_empty() { return Ok(true); }

        if let Some(pos) = expr.find("==") {
            let field = expr[..pos].trim();
            let val = Self::parse_value(expr[pos + 2..].trim());
            let field_val = record.fields.get(field).unwrap_or(&Value::Null);
            return Ok(Self::values_equal(field_val, &val));
        }
        if let Some(pos) = expr.find("!=") {
            let field = expr[..pos].trim();
            let val = Self::parse_value(expr[pos + 2..].trim());
            let field_val = record.fields.get(field).unwrap_or(&Value::Null);
            return Ok(!Self::values_equal(field_val, &val));
        }
        if let Some(pos) = expr.find(">=") {
            let field = expr[..pos].trim();
            let val = Self::parse_value(expr[pos + 2..].trim());
            let field_val = record.fields.get(field).unwrap_or(&Value::Null);
            return Ok(Self::cmp_values(field_val, &val) != std::cmp::Ordering::Less);
        }
        if let Some(pos) = expr.find("<=") {
            let field = expr[..pos].trim();
            let val = Self::parse_value(expr[pos + 2..].trim());
            let field_val = record.fields.get(field).unwrap_or(&Value::Null);
            return Ok(Self::cmp_values(field_val, &val) != std::cmp::Ordering::Greater);
        }
        if let Some(pos) = expr.find('>') {
            if pos > 0 && expr.as_bytes().get(pos.saturating_sub(1)) != Some(&b'>') {
                let field = expr[..pos].trim();
                let val = Self::parse_value(expr[pos + 1..].trim());
                let field_val = record.fields.get(field).unwrap_or(&Value::Null);
                return Ok(Self::cmp_values(field_val, &val) == std::cmp::Ordering::Greater);
            }
        }
        if let Some(pos) = expr.find('<') {
            let field = expr[..pos].trim();
            let val = Self::parse_value(expr[pos + 1..].trim());
            let field_val = record.fields.get(field).unwrap_or(&Value::Null);
            return Ok(Self::cmp_values(field_val, &val) == std::cmp::Ordering::Less);
        }
        if let Some(pos) = expr.find("=~") {
            let field = expr[..pos].trim();
            let pattern = expr[pos + 2..].trim().trim_matches('"');
            if let Ok(re) = regex::Regex::new(pattern) {
                let field_val = record.fields.get(field).unwrap_or(&Value::Null);
                return Ok(re.is_match(&field_val.to_string()));
            }
            return Ok(false);
        }

        let field = expr;
        let field_val = record.fields.get(field).unwrap_or(&Value::Null);
        Ok(match field_val {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty() && s != "0" && s != "false",
            _ => true,
        })
    }

    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x - y).abs() < 0.0001,
            (Value::Int(x), Value::Float(y)) => (*x as f64 - y).abs() < 0.0001,
            (Value::Float(x), Value::Int(y)) => (x - *y as f64).abs() < 0.0001,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            _ => a.to_string() == b.to_string(),
        }
    }

    fn parse_value(s: &str) -> Value {
        let s = s.trim();
        if s == "null" { return Value::Null; }
        if s == "true" { return Value::Bool(true); }
        if s == "false" { return Value::Bool(false); }
        if s.starts_with('"') && s.ends_with('"') && s.len() > 1 {
            return Value::String(s[1..s.len()-1].to_string());
        }
        if s.starts_with('\'') && s.ends_with('\'') && s.len() > 1 {
            return Value::String(s[1..s.len()-1].to_string());
        }
        if let Ok(n) = s.parse::<i64>() { return Value::Int(n); }
        if let Ok(n) = s.parse::<f64>() { return Value::Float(n); }
        Value::String(s.to_string())
    }

    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    if in_quotes {
                        if chars.as_str().starts_with('"') {
                            current.push('"');
                            chars.next();
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                ',' if !in_quotes => {
                    fields.push(current.clone());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        fields.push(current);
        fields
    }

    fn csv_escape(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            let mut out = String::from('"');
            for c in s.chars() {
                if c == '"' { out.push('"'); }
                out.push(c);
            }
            out.push('"');
            out
        } else {
            s.to_string()
        }
    }
}

pub fn pipe_from(args: &[String], mut reader: impl Read, mut writer: impl Write) -> i32 {
    let format = args.get(0).map(|s| s.as_str()).unwrap_or("csv");
    let mut input = String::new();
    if reader.read_to_string(&mut input).is_err() {
        let _ = writeln!(writer, "jesh: from: failed to read input");
        return 1;
    }
    let table = match format {
        "csv" => Table::from_csv(&input),
        "tsv" => Table::from_tsv(&input),
        "json" => Table::from_json(&input),
        _ => { let _ = writeln!(writer, "jesh: from: unknown format '{}'", format); return 1; }
    };
    match table {
        Ok(table) => { let _ = writeln!(writer, "{}", table.to_json_string()); 0 }
        Err(e) => { let _ = writeln!(writer, "jesh: from: {}", e); 1 }
    }
}

pub fn pipe_to(args: &[String], mut reader: impl Read, mut writer: impl Write) -> i32 {
    let format = args.get(0).map(|s| s.as_str()).unwrap_or("table");
    let mut input = String::new();
    if reader.read_to_string(&mut input).is_err() {
        let _ = writeln!(writer, "jesh: to: failed to read input");
        return 1;
    }
    let table = match Table::from_json(&input) {
        Ok(t) => t,
        Err(e) => { let _ = writeln!(writer, "jesh: to: {}", e); return 1; }
    };
    let output = match format {
        "csv" => table.to_csv(),
        "json" => table.to_json_string(),
        "table" => table.to_table(),
        _ => { let _ = writeln!(writer, "jesh: to: unknown format '{}'", format); return 1; }
    };
    let _ = write!(writer, "{}", output);
    0
}

pub fn pipe_where(args: &[String], mut reader: impl Read, mut writer: impl Write) -> i32 {
    let expr = args.join(" ");
    if expr.is_empty() {
        let _ = writeln!(writer, "jesh: where: missing expression");
        return 1;
    }
    let mut input = String::new();
    if reader.read_to_string(&mut input).is_err() {
        let _ = writeln!(writer, "jesh: where: failed to read input");
        return 1;
    }
    let table = match Table::from_json(&input) {
        Ok(t) => t,
        Err(e) => { let _ = writeln!(writer, "jesh: where: {}", e); return 1; }
    };
    match table.where_filter(&expr) {
        Ok(filtered) => { let _ = writeln!(writer, "{}", filtered.to_json_string()); 0 }
        Err(e) => { let _ = writeln!(writer, "jesh: where: {}", e); 1 }
    }
}

pub fn pipe_select(args: &[String], mut reader: impl Read, mut writer: impl Write) -> i32 {
    if args.is_empty() {
        let _ = writeln!(writer, "jesh: select: missing columns");
        return 1;
    }
    let mut input = String::new();
    if reader.read_to_string(&mut input).is_err() {
        let _ = writeln!(writer, "jesh: select: failed to read input");
        return 1;
    }
    let table = match Table::from_json(&input) {
        Ok(t) => t,
        Err(e) => { let _ = writeln!(writer, "jesh: select: {}", e); return 1; }
    };
    let selected = table.select(args);
    let _ = writeln!(writer, "{}", selected.to_json_string());
    0
}

pub fn pipe_sort_by(args: &[String], mut reader: impl Read, mut writer: impl Write) -> i32 {
    if args.is_empty() {
        let _ = writeln!(writer, "jesh: sort-by: missing column");
        return 1;
    }
    let desc = args.get(0).map(|s| s.as_str()) == Some("--desc");
    let col = if desc { args.get(1).cloned() } else { args.get(0).cloned() };
    let col = match col {
        Some(c) => c,
        None => { let _ = writeln!(writer, "jesh: sort-by: missing column"); return 1; }
    };
    let mut input = String::new();
    if reader.read_to_string(&mut input).is_err() {
        let _ = writeln!(writer, "jesh: sort-by: failed to read input");
        return 1;
    }
    let table = match Table::from_json(&input) {
        Ok(t) => t,
        Err(e) => { let _ = writeln!(writer, "jesh: sort-by: {}", e); return 1; }
    };
    let sorted = table.sort_by_col(&col, desc);
    let _ = writeln!(writer, "{}", sorted.to_json_string());
    0
}

pub fn is_semantic_builtin(cmd: &str) -> bool {
    matches!(cmd, "from" | "to" | "where" | "select" | "sort-by")
}

pub fn run_semantic_pipeline(cmd: &str, args: &[String], reader: impl Read + Send + 'static, writer: impl Write + Send + 'static) -> i32 {
    match cmd {
        "from" => pipe_from(args, reader, writer),
        "to" => pipe_to(args, reader, writer),
        "where" => pipe_where(args, reader, writer),
        "select" => pipe_select(args, reader, writer),
        "sort-by" => pipe_sort_by(args, reader, writer),
        _ => 1,
    }
}
