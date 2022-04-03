use std::fmt::{Display, Formatter};

use crate::*;
use crate::file::{FileMan, FileManResult};

use chrono::Local;

pub type ConsoleResult<T> = Result<T, ()>;

#[macro_export]
macro_rules! log {
    ($kind:ident, $title:expr $(, $desc:expr)*) => {
        {
            ConsoleLog {
                kind: ConsoleLogKind::$kind,
                title: Box::new($title),
                descs: vec![
                    $(Box::new($desc),)*
                ]
            }
        }
    };
}

#[macro_export]
macro_rules! translate {
    (translator => $log:expr, lang => $lang:expr, $($log_key:pat => {$($lang_key:pat => $value:expr,)*},)*) => {
        match $log {
            $(
                $log_key => {
                    match $lang {
                        $($lang_key => $value.to_string(),)+
                    }
                },
            )+
        }
    };
}

pub trait ConsoleLogger: Clone + PartialEq {
    fn get_log(&self) -> ConsoleLog;
}

#[derive(Clone, PartialEq)]
pub enum TranslationResult {
    Success(String),
    UnknownLanguage
}

pub trait ConsoleLogTranslator: Send {
    fn translate(&self, lang_name: &str) -> TranslationResult;
}

#[derive(Clone, PartialEq)]
pub enum ConsoleLogKind {
    Error,
    Warning,
    Note,
}

impl ConsoleLogKind {
    fn get_log_color_num(&self) -> usize {
        return match self {
            ConsoleLogKind::Error => 31,
            ConsoleLogKind::Warning => 33,
            ConsoleLogKind::Note => 34,
        };
    }

    fn get_log_kind_name(&self) -> String {
        let s = match self {
            ConsoleLogKind::Error => "err",
            ConsoleLogKind::Warning => "warn",
            ConsoleLogKind::Note => "note",
        };

        return s.to_string();
    }
}

pub struct ConsoleLog {
    pub kind: ConsoleLogKind,
    pub title: Box<dyn ConsoleLogTranslator>,
    pub descs: Vec<Box<dyn ConsoleLogTranslator>>,
}

#[derive(Clone, PartialEq)]
pub enum LogFileKind {
    TextLines(Vec<String>),
    ConsoleLogs,
}

#[derive(Clone, PartialEq)]
pub struct LogFile {
    kind: LogFileKind,
    output_path: String,
}

impl LogFile {
    pub fn new(kind: LogFileKind, output_path: String) -> LogFile {
        return LogFile {
            kind: kind,
            output_path: output_path,
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleLogLimit {
    NoLimit,
    Limited(usize),
}

impl Display for ConsoleLogLimit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConsoleLogLimit::NoLimit => "[no limit]".to_string(),
            ConsoleLogLimit::Limited(limit_count) => limit_count.to_string(),
        };

        return write!(f, "{}", s);
    }
}

pub struct Console {
    lang: String,
    log_list: Vec<ConsoleLog>,
    log_limit: ConsoleLogLimit,
    pub ignore_logs: bool,
}

impl Console {
    pub fn new(lang: String, log_limit: ConsoleLogLimit) -> Console {
        return Console {
            lang: lang,
            log_list: Vec::new(),
            log_limit: log_limit,
            ignore_logs: false,
        };
    }

    pub fn append_log(&mut self, log: ConsoleLog) {
        if !self.ignore_logs {
            self.log_list.push(log);
        }
    }

    pub fn clear(&mut self) {
        self.log_list.clear();
    }

    pub fn pop_log(&mut self) {
        if self.log_list.len() > 0 {
            self.log_list.pop();
        }
    }

    pub fn output(&self, log_files: Vec<LogFile>) {
        let mut cons_log_lines = Vec::<String>::new();
        self.print_all(&mut cons_log_lines);

        match self.write_all(log_files, cons_log_lines) {
            Ok(()) => (),
            Err(_) => println!("{}", Console::format_log_file_writing_failure_log()),
        };
    }

    fn write_all(&self, log_files: Vec<LogFile>, cons_log_lines: Vec<String>) -> FileManResult<()> {
        let header = vec![
            "--- Log File ---",
            "",
            &format!(" * created at {}", Local::now()),
            " * generated by cons-util",
        ].join("\n");

        for each_file_log in log_files {
            let lines = match &each_file_log.kind {
                LogFileKind::TextLines(lines) => lines,
                LogFileKind::ConsoleLogs => &cons_log_lines,
            };

            let output_content = header.clone() + "\n\n" + &lines.join("\n");

            // fix: -> write_lines()
            FileMan::write_all(&each_file_log.output_path, &output_content)?;
        }

        return Ok(());
    }

    fn print_all(&self, log_lines: &mut Vec<String>) {
        // note: ログ数制限のチェック
        let limit_num = match &self.log_limit {
            ConsoleLogLimit::NoLimit => -1i32,
            ConsoleLogLimit::Limited(v) => *v as i32,
        };

        let mut log_count = 0;

        for each_log in &self.log_list {
            if limit_num != -1 && log_count + 1 > limit_num as i32 {
                self.print(&log!(Note, InternalTranslator::LogLimitExceeded { log_limit: self.log_limit.clone() }), &mut Vec::new());
                break;
            }

            self.print(each_log, log_lines);
            log_count += 1;
        }
    }

    fn print(&self, log: &ConsoleLog, log_lines: &mut Vec<String>) {
        let title_color = log.kind.get_log_color_num();
        let kind_name = log.kind.get_log_kind_name();

        let title = match log.title.translate(&self.lang) {
            TranslationResult::Success(v) => v,
            TranslationResult::UnknownLanguage => {
                println!("{}", Console::format_unknown_language_log());
                println!();
                return;
            },
        };

        println!("{}", Console::format_title(Some(title_color), &kind_name, &title));
        log_lines.push(Console::format_title(None, &kind_name, &title));

        for each_desc_result in &log.descs {
            let each_desc = match each_desc_result.translate(&self.lang) {
                TranslationResult::Success(v) => v,
                TranslationResult::UnknownLanguage => {
                    println!("{}", Console::format_unknown_language_log());
                    println!();
                    return;
                },
            };

            println!("{}", each_desc);
            log_lines.push(each_desc);
        }

        println!();
        log_lines.push(String::new());
    }

    fn format_unknown_language_log() -> String {
        let err_log_kind = ConsoleLogKind::Error;
        return Console::format_title(Some(err_log_kind.get_log_color_num()), &err_log_kind.get_log_kind_name(), "unknown language");
    }

    fn format_log_file_writing_failure_log() -> String {
        let err_log_kind = ConsoleLogKind::Error;
        return Console::format_title(Some(err_log_kind.get_log_color_num()), &err_log_kind.get_log_kind_name(), "log file writing failure");
    }

    fn format_title(color: Option<usize>, kind: &str, title: &str) -> String {
        let (color_begin, color_end) = match color {
            Some(v) => (format!("\x1b[{}m", v), "\x1b[m".to_string()),
            None => (String::new(), String::new()),
        };

        return format!("{}[{}]{} {}", color_begin, kind, color_end, title);
    }
}