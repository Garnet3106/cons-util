use {
    // cons-util-derive で展開されるコードへの対応
    crate as cons_util,
    crate::cons::*,

    std::{
        env::current_dir,
        fmt::{
            Display,
            Formatter,
        },
        fs::*,
        io::*,
        path::PathBuf,
        time::SystemTime,
        result::Result,
    },
};

pub type FileManResult<T> = Result<T, FileManLog>;

#[derive(Clone, cons_util_derive::ConsoleLogTranslator, Debug, PartialEq)]
pub enum FileManLog {
    #[translate(
        kind = "E",
        en = "expected file path not directory path",
        ja = "ディレクトリパスでなくファイルパスが必要です",
    )]
    ExpectedFilePathNotDirectoryPath,

    #[translate(
        kind = "E",
        en = "failed to get current directory",
        ja = "カレントディレクトリの取得に失敗しました",
    )]
    FailedToGetCurrentDirectory,

    #[translate(
        kind = "E",
        en = "failed to open file\n\tpath: {path}",
        ja = "ファイルのオープンに失敗しました\n\tパス: {path}",
    )]
    FailedToOpenFile { path: String },

    #[translate(
        kind = "E",
        en = "failed to open file or directory\n\tpath: {path}",
        ja = "ファイルもしくはディレクトリのオープンに失敗しました\n\tパス: {path}",
    )]
    FailedToOpenFileOrDirectory { path: String },

    #[translate(
        kind = "E",
        en = "failed to read file\n\tpath: {path}",
        ja = "ファイルの読み込みに失敗しました\n\tパス: {path}",
    )]
    FailedToReadFile { path: String },

    #[translate(
        kind = "E",
        en = "failed to write file\n\tpath: {path}",
        ja = "ファイルの書き込みに失敗しました\n\tパス: {path}",
    )]
    FailedToWriteFile { path: String },

    #[translate(
        kind = "E",
        en = "metadata is not available on this platform",
        ja = "このプラットフォームでは属性が利用できません",
    )]
    MetadataIsNotAvailableOnThisPlatform,

    #[translate(
        kind = "E",
        en = "path does not exist\n\tpath: {path}",
        ja = "パスが存在しません\n\tパス: {path}",
    )]
    PathDoesNotExist { path: String },
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
            Err(_) => Err(FileManLog::MetadataIsNotAvailableOnThisPlatform),
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
            Err(FileManLog::ExpectedFilePathNotDirectoryPath)
        };
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}
