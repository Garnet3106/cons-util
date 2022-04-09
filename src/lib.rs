pub mod cons;
pub mod file;
pub mod js;

use crate::cons::*;

pub trait ConsoleResultConsumption<T> {
    fn consume(self, cons: &mut Console) -> ConsoleResult<T>;
}

impl<T, E: ConsoleLogger> ConsoleResultConsumption<T> for Result<T, E> {
    fn consume(self, cons: &mut Console) -> ConsoleResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                cons.append_log(e.get_log());
                Err(())
            },
        }
    }
}

enum InternalLanguage {
    English,
    Japanese,
}

impl InternalLanguage {
    pub fn from(s: &str) -> Option<InternalLanguage> {
        let v = match s {
            "en" => InternalLanguage::English,
            "ja" => InternalLanguage::Japanese,
            _ => return None,
        };

        return Some(v);
    }
}

#[derive(Clone, PartialEq)]
enum InternalTranslator {
    // note: log titles
    ExpectedFilePathNotDirectoryPath,
    FailedToGetCurrentDirectory,
    FailedToOpenFile,
    FailedToOpenFileOrDirectory,
    FailedToReadFile,
    FailedToWriteFile,
    LogLimitExceeded { log_limit: ConsoleLogLimit },
    MetadataIsNotAvailableOnThisPlatform,
    PathDoesNotExist,
    // note: descriptions
    PathDescription { path: String },
}

impl ConsoleLogTranslator for InternalTranslator {
    fn translate(&self, lang_name: &str) -> TranslationResult {
        let lang = match InternalLanguage::from(lang_name) {
            Some(v) => v,
            None => return TranslationResult::UnknownLanguage,
        };

        let s = translate!{
            translator => self,
            lang => lang,
            // note: log titles
            InternalTranslator::ExpectedFilePathNotDirectoryPath => {
                InternalLanguage::English => "expected file path not directory path",
                InternalLanguage::Japanese => "ディレクトリパスでなくファイルパスが必要です",
            },
            InternalTranslator::FailedToGetCurrentDirectory => {
                InternalLanguage::English => "failed to get current directory",
                InternalLanguage::Japanese => "カレントディレクトリの取得に失敗しました",
            },
            InternalTranslator::FailedToOpenFile => {
                InternalLanguage::English => "failed to open file",
                InternalLanguage::Japanese => "ファイルのオープンに失敗しました",
            },
            InternalTranslator::FailedToOpenFileOrDirectory => {
                InternalLanguage::English => "failed to open file or directory",
                InternalLanguage::Japanese => "ファイルもしくはディレクトリのオープンに失敗しました",
            },
            InternalTranslator::FailedToReadFile => {
                InternalLanguage::English => "failed to read file",
                InternalLanguage::Japanese => "ファイルの読み込みに失敗しました",
            },
            InternalTranslator::FailedToWriteFile => {
                InternalLanguage::English => "failed to write file",
                InternalLanguage::Japanese => "ファイルの書き込みに失敗しました",
            },
            InternalTranslator::LogLimitExceeded { log_limit } => {
                InternalLanguage::English => format!("log limit {} exceeded", log_limit),
                InternalLanguage::Japanese => format!("ログ制限 {} を超過しました", log_limit),
            },
            InternalTranslator::MetadataIsNotAvailableOnThisPlatform => {
                InternalLanguage::English => "metadata is not available on this platform",
                InternalLanguage::Japanese => "このプラットフォームでは属性が利用できません",
            },
            InternalTranslator::PathDoesNotExist => {
                InternalLanguage::English => "path does not exist",
                InternalLanguage::Japanese => "パスが存在しません",
            },
            // note: descriptions
            InternalTranslator::PathDescription { path } => {
                InternalLanguage::English => format!("path:\t{}", path),
                InternalLanguage::Japanese => format!("パス:\t{}", path),
            },
        };

        return TranslationResult::Success(s);
    }
}
