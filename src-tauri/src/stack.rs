use crate::model::StackFrame;
pub trait StackTraceParser {
    fn parse(&self, raw: &str) -> Vec<StackFrame>;
}
pub struct NodeParser;
impl StackTraceParser for NodeParser {
    fn parse(&self, raw: &str) -> Vec<StackFrame> {
        let re =
            regex::Regex::new(r"^\s*at\s+(?:(.*?)\s+\()?(.+?):(\d+)(?::(\d+))?\)?\s*$").unwrap();
        raw.lines()
            .filter_map(|line| {
                let c = re.captures(line)?;
                let file = c[2].to_string();
                Some(StackFrame {
                    index: 0,
                    raw: line.into(),
                    function_name: c.get(1).map(|s| s.as_str().into()),
                    file_path: file.clone(),
                    line: c[3].parse().ok()?,
                    column: c.get(4).and_then(|s| s.as_str().parse().ok()),
                    language: if file.ends_with(".ts") || file.ends_with(".tsx") {
                        "typescript"
                    } else {
                        "javascript"
                    }
                    .into(),
                    resolved: false,
                })
            })
            .enumerate()
            .map(|(i, mut f)| {
                f.index = i;
                f
            })
            .collect()
    }
}
