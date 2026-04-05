use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: Regex = Regex::new(r"(?i)\b(SELECT|FROM|WHERE|INSERT|INTO|VALUES|UPDATE|SET|DELETE|JOIN|LEFT|RIGHT|INNER|OUTER|ON|GROUP BY|ORDER BY|HAVING|LIMIT|OFFSET|AS|ASC|DESC|AND|OR|NOT|NULL|IS|CREATE|TABLE|DROP|ALTER|INDEX|PRIMARY KEY|FOREIGN KEY)\b").unwrap();
    static ref STRINGS: Regex = Regex::new(r"(?m)('.*?'|[^']*)").unwrap(); // Simple string matcher
}

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE", "SET", 
    "DELETE", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON", "GROUP BY", 
    "ORDER BY", "HAVING", "LIMIT", "OFFSET", "AS", "ASC", "DESC", "AND", 
    "OR", "NOT", "NULL", "IS", "CREATE", "TABLE", "DROP", "ALTER", "INDEX", 
    "PRIMARY KEY", "FOREIGN KEY"
];

pub struct SqlEditor {
    pub lines: Vec<String>,
    pub cursor_y: usize,
    pub cursor_x: usize,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub suggestion_active: bool,
    pub dynamic_keywords: Vec<String>,
}

impl SqlEditor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_y: 0,
            cursor_x: 0,
            suggestions: vec![],
            selected_suggestion: 0,
            suggestion_active: false,
            dynamic_keywords: vec![],
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_y];
        if self.cursor_x <= line.len() {
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
        }
        self.update_suggestions();
    }

    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cursor_y];
        let rest = line.split_off(self.cursor_x);
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, rest);
        self.cursor_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            self.cursor_x -= 1;
            line.remove(self.cursor_x);
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
        }
        self.update_suggestions();
    }

    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    pub fn move_right(&mut self) {
        let len = self.lines[self.cursor_y].len();
        if self.cursor_x < len {
            self.cursor_x += 1;
        } else if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let len = self.lines[self.cursor_y].len();
            if self.cursor_x > len {
                self.cursor_x = len;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            let len = self.lines[self.cursor_y].len();
            if self.cursor_x > len {
                self.cursor_x = len;
            }
        }
    }

    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    fn get_current_word(&self) -> Option<(usize, usize, String)> {
        let line = &self.lines[self.cursor_y];
        if self.cursor_x == 0 || self.cursor_x > line.len() { return None; }
        
        let mut start_x = self.cursor_x;
        for c in line[..self.cursor_x].chars().rev() {
            if !c.is_alphabetic() && c != '_' {
                break;
            }
            start_x -= c.len_utf8();
        }
        if start_x == self.cursor_x {
            return None;
        }
        let word = line[start_x..self.cursor_x].to_uppercase();
        Some((start_x, self.cursor_x, word))
    }

    pub fn update_suggestions(&mut self) {
        if let Some((_, _, current_word)) = self.get_current_word() {
            let mut matches: Vec<String> = SQL_KEYWORDS
                .iter()
                .filter(|&&kw| kw.starts_with(&current_word) && kw != current_word)
                .map(|kw| kw.to_string())
                .collect();
                
            let lower_current = current_word.to_lowercase();
            let dyn_matches: Vec<String> = self.dynamic_keywords
                .iter()
                .filter(|kw| kw.to_lowercase().starts_with(&lower_current) && kw.to_lowercase() != lower_current)
                .cloned()
                .collect();
                
            matches.extend(dyn_matches);
            matches.truncate(6);

            if matches.is_empty() {
                self.suggestion_active = false;
                self.suggestions.clear();
            } else {
                self.suggestion_active = true;
                self.suggestions = matches;
                if self.selected_suggestion >= self.suggestions.len() {
                    self.selected_suggestion = 0;
                }
            }
        } else {
            self.suggestion_active = false;
            self.suggestions.clear();
        }
    }

    pub fn apply_suggestion(&mut self) {
        if !self.suggestion_active || self.suggestions.is_empty() {
            return;
        }
        if let Some((start_x, end_x, _)) = self.get_current_word() {
            let suggestion = &self.suggestions[self.selected_suggestion];
            let mut line = self.lines[self.cursor_y].clone();
            line.replace_range(start_x..end_x, suggestion);
            self.lines[self.cursor_y] = line;
            self.cursor_x = start_x + suggestion.len(); // jump to end of suggestion
            // Add trailing space
            let line = &mut self.lines[self.cursor_y];
            line.insert(self.cursor_x, ' ');
            self.cursor_x += 1;
        }
        self.suggestion_active = false;
        self.suggestions.clear();
    }

    pub fn highlight<'a>(&'a self) -> Vec<Line<'a>> {
        self.lines.iter().map(|line| {
            // A very simple regex highlighting logic snippet
            // In a real app we'd build a proper lexer
            let mut spans = vec![];
            
            // Iterate over matches and non-matches of split_regex is cleaner by just searching keywords over the whole string.
            // Better simpler highlighter:
            let mut last_end = 0;
            for mat in KEYWORDS.find_iter(line) {
                if mat.start() > last_end {
                    spans.push(Span::raw(&line[last_end..mat.start()]));
                }
                spans.push(Span::styled(&line[mat.start()..mat.end()], Style::default().fg(Color::LightBlue)));
                last_end = mat.end();
            }
            if last_end < line.len() {
                spans.push(Span::raw(&line[last_end..]));
            }

            Line::from(spans)
        }).collect()
    }
}
