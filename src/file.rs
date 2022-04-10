use crate::*;
use crate::cons::*;

use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::fs::*;
use std::io::*;
use std::path::PathBuf;
use std::time::SystemTime;
use std::result::Result;

pub type FileManResult<T> = Result<T, FileManLog>;

#[derive(Clone, Debug, PartialEq)]
pub enum FileManLog {
    ExpectedFilePathNotDirectoryPath { path: String },
    FailedToGetCurrentDirectory,
    FailedToOpenFile { path: String },
    FailedToOpenFileOrDirectory { path: String },
    FailedToReadFile { path: String },
    FailedToWriteFile { path: String },
    LogLimitExceeded { log_limit: ConsoleLogLimit },
    MetadataIsNotAvailableOnThisPlatform { path: String },
    PathDoesNotExist { path: String },
}

impl ConsoleLogger for FileManLog {
    fn get_log(&self) -> ConsoleLog {
        match self {
            FileManLog::ExpectedFilePathNotDirectoryPath { path } => log!(Error, InternalTranslator::ExpectedFilePathNotDirectoryPath, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::FailedToGetCurrentDirectory => log!(Error, InternalTranslator::FailedToGetCurrentDirectory),
            FileManLog::FailedToOpenFile { path } => log!(Error, InternalTranslator::FailedToOpenFile, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::FailedToOpenFileOrDirectory { path } => log!(Error, InternalTranslator::FailedToOpenFileOrDirectory, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::FailedToReadFile { path } => log!(Error, InternalTranslator::FailedToReadFile, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::FailedToWriteFile { path } => log!(Error, InternalTranslator::FailedToWriteFile, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::LogLimitExceeded { log_limit } => log!(Error, InternalTranslator::LogLimitExceeded { log_limit: log_limit.clone() }),
            FileManLog::MetadataIsNotAvailableOnThisPlatform { path } => log!(Error, InternalTranslator::MetadataIsNotAvailableOnThisPlatform, InternalTranslator::PathDescription { path: path.clone() }),
            FileManLog::PathDoesNotExist { path } => log!(Error, InternalTranslator::PathDoesNotExist, InternalTranslator::PathDescription { path: path.clone() }),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct FilePath(String, PathBuf);

impl FilePath {
    pub fn new(path: String) -> FilePath {
        return FilePath(path.clone(), PathBuf::from(path));
    }

    pub fn from(path: PathBuf) -> FilePath {
        return FilePath(path.clone().into_os_string().into_string().unwrap(), path);
    }

    pub fn to_absolute(&self) -> FileManResult<FilePath> {
        let curr_dir_path_obj = match current_dir() {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToGetCurrentDirectory),
        };

        return Ok(FilePath::from(curr_dir_path_obj.join(&self.0)));
    }

    pub fn exists(&self) -> bool {
        return self.1.exists();
    }

    pub fn is_dir(&self) -> bool {
        return self.1.is_dir();
    }

    pub fn is_file(&self) -> bool {
        return !self.1.is_dir();
    }

    pub fn is_same_as(&self, path: &FilePath) -> FileManResult<bool> {
        return match same_file::is_same_file(&self.0, &path.0) {
            Ok(v) => Ok(v),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory { path: format!("{}; {}", self.0, path.0) }),
        };
    }

    pub fn join(&self, rel_path: &FilePath) -> FileManResult<FilePath> {
        let joined_path_obj = self.1.join(&rel_path.0);

        return match joined_path_obj.canonicalize() {
            Ok(v) => Ok(FilePath::from(v)),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory { path: joined_path_obj.to_str().unwrap().to_string() }),
        };
    }

    pub fn last_modified(&self) -> FileManResult<SystemTime> {
        let metadata = self.metadata()?;

        return match metadata.modified() {
            Ok(time) => Ok(time),
            Err(_) => Err(FileManLog::MetadataIsNotAvailableOnThisPlatform {
                path: self.0.clone(),
            }),
        };
    }

    pub fn metadata(&self) -> FileManResult<Metadata> {
        return match metadata(&self.0) {
            Ok(v) => Ok(v),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory {
                path: self.0.clone(),
            }),
        };
    }

    pub fn parent_dir(&self) -> FileManResult<Option<FilePath>> {
        if !self.exists() {
            return Err(FileManLog::PathDoesNotExist { path: self.0.clone() });
        }

        let parent_path = match self.1.parent() {
            Some(v) => Some(FilePath::from(PathBuf::from(v))),
            None => None,
        };

        return Ok(parent_path);
    }

    pub fn read(&self) -> FileManResult<String> {
        self.ensure_exists()?;
        self.ensure_be_file()?;

        let content = match std::fs::read_to_string(&self.0) {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToReadFile { path: self.0.clone() }),
        };

        return Ok(content);
    }

    pub fn read_bytes(&self) -> FileManResult<Vec<u8>> {
        self.ensure_exists()?;
        self.ensure_be_file()?;

        let mut reader = match File::open(&self.0) {
            Ok(v) => BufReader::new(v),
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: self.0.clone() }),
        };

        let mut bytes = Vec::<u8>::new();
        let mut buf = [0; 4];

        loop {
            match reader.read(&mut buf) {
                Ok(v) => {
                    match v {
                        0 => break,
                        n => {
                            let buf = &buf[..n];
                            bytes.append(&mut buf.to_vec());
                        }
                    }
                },
                Err(_) => return Err(FileManLog::FailedToReadFile { path: self.0.clone() }),
            }
        }

        return Ok(bytes);
    }

    pub fn read_lines(&self) -> FileManResult<Vec<String>> {
        self.ensure_exists()?;
        self.ensure_be_file()?;

        let reader = match File::open(&self.0) {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: self.0.clone() }),
        };

        let mut lines = Vec::<String>::new();

        for each_line in BufReader::new(reader).lines() {
            lines.push(
                match each_line {
                    Ok(v) => v,
                    Err(_) => return Err(FileManLog::FailedToReadFile { path: self.0.clone() }),
                }
            );
        }

        return Ok(lines);
    }

    pub fn change_extension(&self, new_ext: &str) -> String {
        let split_path: Vec<&str> = self.0.split(".").collect();

        // 拡張子がついていない場合は新しく付け足す
        if split_path.len() < 2 {
            return self.0.clone() + "." + new_ext;
        }

        let old_ext_raw: Vec<&str> = split_path[split_path.len() - 1..split_path.len()].to_vec();
        let old_ext = old_ext_raw.get(0).unwrap();

        return self.0[0..self.0.len() - old_ext.len()].to_string() + new_ext;
    }

    pub fn create_file(&self) -> FileManResult<File> {
        return match File::create(&self.0) {
            Ok(v) => Ok(v),
            Err(_) => Err(FileManLog::FailedToOpenFile { path: self.0.clone() }),
        };
    }

    pub fn write(&self, content: &String) -> FileManResult<()> {
        return self.write_bytes(content.as_bytes());
    }

    pub fn write_bytes(&self, bytes: &[u8]) -> FileManResult<()> {
        let mut file = self.create_file()?;

        match file.write_all(bytes) {
            Err(_) => return Err(FileManLog::FailedToWriteFile { path: self.0.clone() }),
            Ok(v) => v,
        };

        return Ok(());
    }

    pub fn ensure_exists(&self) -> FileManResult<()> {
        return if self.exists() {
            Ok(())
        } else {
            Err(FileManLog::PathDoesNotExist { path: self.0.clone() })
        };
    }

    pub fn ensure_be_file(&self) -> FileManResult<()> {
        return if self.is_file() {
            Ok(())
        } else {
            Err(FileManLog::ExpectedFilePathNotDirectoryPath { path: self.0.clone() })
        };
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}
