pub fn truncate_middle(s: &str, keep: usize) -> String {
    if s.chars().count() <= keep { return s.to_string(); }
    let half = keep / 2;
    let start: String = s.chars().take(half).collect();
    let end: String = s.chars().rev().take(half).collect::<String>().chars().rev().collect();
    format!("{}...{}", start, end)
}