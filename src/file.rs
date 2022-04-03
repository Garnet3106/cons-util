use crate::*;
use crate::cons::*;

use std::env::current_dir;
use std::fs::*;
use std::io::*;
use std::path::Path;
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

pub struct FileMan {}

impl FileMan {
    pub fn abs(rel_path: &str) -> FileManResult<Box<Path>> {
        let rel_path_obj = Path::new(rel_path);

        let curr_dir_path_obj = match current_dir() {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToGetCurrentDirectory {}),
        };

        return Ok(Box::from(curr_dir_path_obj.join(rel_path_obj)));
    }

    pub fn exists(path: &str) -> bool {
        return Path::new(path).exists();
    }

    pub fn is_dir(path: &str) -> bool {
        return Path::new(path).is_dir();
    }

    pub fn is_same(path1: &str, path2: &str) -> FileManResult<bool> {
        return match same_file::is_same_file(path1, path2) {
            Ok(v) => Ok(v),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory { path: format!("{}; {}", path1, path2) }),
        };
    }

    pub fn join_path(orig_path: &str, rel_path: &str) -> FileManResult<Box<Path>> {
        let orig_path_obj = Path::new(orig_path);
        let rel_path_obj = Path::new(rel_path);
        let joined_path_obj = orig_path_obj.join(rel_path_obj);

        return match joined_path_obj.canonicalize() {
            Ok(v) => Ok(Box::from(v)),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory { path: joined_path_obj.to_str().unwrap().to_string() }),
        };
    }

    pub fn last_modified(path: &str) -> FileManResult<SystemTime> {
        let metadata = FileMan::metadata(path)?;

        return match metadata.modified() {
            Ok(time) => Ok(time),
            Err(_) => Err(FileManLog::MetadataIsNotAvailableOnThisPlatform {
                path: path.to_string(),
            }),
        };
    }

    pub fn metadata(path: &str) -> FileManResult<Metadata> {
        return match metadata(path) {
            Ok(v) => Ok(v),
            Err(_) => Err(FileManLog::FailedToOpenFileOrDirectory {
                path: path.to_string(),
            }),
        };
    }

    pub fn parent_dir(path: &str) -> FileManResult<Option<Box<Path>>> {
        if !FileMan::exists(path) {
            return Err(FileManLog::PathDoesNotExist { path: path.to_string() });
        }

        let parent_path = match Path::new(path).parent() {
            Some(v) => v,
            None => return Ok(None),
        };

        return Ok(Some(Box::from(parent_path)));
    }

    pub fn read_all(path: &str) -> FileManResult<String> {
        if !FileMan::exists(&path) {
            return Err(FileManLog::PathDoesNotExist { path: path.to_string() });
        }

        if FileMan::is_dir(&path) {
            return Err(FileManLog::ExpectedFilePathNotDirectoryPath { path: path.to_string() });
        }

        let content = match std::fs::read_to_string(path) {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToReadFile { path: path.to_string() }),
        };

        return Ok(content);
    }

    pub fn read_all_bytes(path: &str) -> FileManResult<Vec<u8>> {
        if !FileMan::exists(&path) {
            return Err(FileManLog::PathDoesNotExist { path: path.to_string() });
        }

        if FileMan::is_dir(&path) {
            return Err(FileManLog::ExpectedFilePathNotDirectoryPath { path: path.to_string() });
        }

        let mut reader = match File::open(path) {
            Ok(v) => BufReader::new(v),
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: path.to_string() }),
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
                Err(_) => return Err(FileManLog::FailedToReadFile { path: path.to_string() }),
            }
        }

        return Ok(bytes);
    }

    pub fn read_lines(path: &str) -> FileManResult<Vec<String>> {
        if !FileMan::exists(&path) {
            return Err(FileManLog::PathDoesNotExist { path: path.to_string() });
        }

        if FileMan::is_dir(&path) {
            return Err(FileManLog::ExpectedFilePathNotDirectoryPath { path: path.to_string() });
        }

        let reader = match File::open(path) {
            Ok(v) => v,
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: path.to_string() }),
        };

        let mut lines = Vec::<String>::new();

        for each_line in BufReader::new(reader).lines() {
            lines.push(match each_line {
                Ok(v) => v,
                Err(_) => return Err(FileManLog::FailedToReadFile { path: path.to_string() }),
            });
        }

        return Ok(lines);
    }

    pub fn reext(path: &str, new_ext: &str) -> String {
        let split_path: Vec<&str> = path.split(".").collect();

        // 拡張子がついていない場合は新しく付け足す
        if split_path.len() < 2 {
            return path.to_string() + "." + new_ext;
        }

        let old_ext_raw: Vec<&str> = split_path[split_path.len() - 1..split_path.len()].to_vec();
        let old_ext = old_ext_raw.get(0).unwrap();

        return path[0..path.len() - old_ext.len()].to_string() + new_ext;
    }

    pub fn write_all(path: &str, content: &String) -> FileManResult<()> {
        let mut file = match File::create(path) {
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: path.to_string() }),
            Ok(v) => v,
        };

        match file.write_all(content.as_bytes()) {
            Err(_) => return Err(FileManLog::FailedToWriteFile { path: path.to_string() }),
            Ok(v) => v,
        };

        return Ok(());
    }

    pub fn write_all_bytes(path: &str, bytes: &Vec<u8>) -> FileManResult<()> {
        let mut file = match File::create(path) {
            Err(_) => return Err(FileManLog::FailedToOpenFile { path: path.to_string() }),
            Ok(v) => v,
        };

        match file.write_all(bytes) {
            Err(_) => return Err(FileManLog::FailedToWriteFile { path: path.to_string() }),
            Ok(v) => v,
        };

        return Ok(());
    }
}
