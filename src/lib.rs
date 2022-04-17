pub mod cons;
pub mod file;
pub mod js;

use {
    crate as cons_util,
    crate::cons::*,
};

pub trait ConsoleResultConsumption<T> {
    fn consume(self, cons: &mut Console) -> ConsoleResult<T>;
}

impl<T, E: ConsoleLogTranslator> ConsoleResultConsumption<T> for Result<T, E> {
    fn consume(self, cons: &mut Console) -> ConsoleResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                cons.append_log(e.translate(&cons.get_lang()));
                Err(())
            },
        }
    }
}

#[derive(Clone, cons_util_derive::ConsoleLogTranslator, Debug, PartialEq)]
pub enum InternalLog {
    #[translate(
        kind = "E",
        en = "log limit {log_limit} exceeded",
        ja = "ログ制限 {log_limit} を超過しました",
    )]
    LogLimitExceeded { log_limit: ConsoleLogLimit },
}
