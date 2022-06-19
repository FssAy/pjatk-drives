use super::*;

const FILE_ITEM: &'static str = include_str!("file-item.html");

/// Representation of the sftp file entity.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Listing {
    Dir(String),
    File { name: String, size: u64 },
    Unknown,
}

impl Listing {
    #[allow(dead_code)]
    #[deprecated]
    /// Create new listing entity with file metadata
    pub fn new_with_stat(data: (PathBuf, FileStat)) -> (Listing, FileStat) {
        let stat = data.1.clone();
        let listing = Listing::from(data);
        (listing, stat)
    }
}

impl From<(PathBuf, FileStat)> for Listing {
    fn from(data: (PathBuf, FileStat)) -> Self {
        let file = data.1;
        let name = data
            .0
            .file_name()
            .map(|x| format!("{}", x.to_string_lossy()))
            .unwrap_or("<<invalid>>".into());

        match file.file_type() {
            FileType::Directory => Self::Dir(format!("{name}")),
            FileType::RegularFile => Self::File {
                name,
                size: file.size.unwrap_or(0),
            },
            _ => Self::Unknown,
        }
    }
}

/// Representation of the sftp file entity in the html format
pub struct ListingHTML {
    pub data: String,
}

impl ListingHTML {
    /// get css class for the tag based on the file extension.
    ///
    /// Example: <br>
    /// Files with ".exe" extension will have different css styles then files with ".png" extension
    fn css_type_from_file_name(name: &String) -> &'static str {
        let file = Path::new(name);
        if let Some(extension) = file.extension() {
            match extension
                .to_str()
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "ai" | "bmp" | "gif" | "ico" | "jpeg" | "jpg" | "png" | "svg" | "tif" | "tiff" => {
                    "fa-file-image"
                }
                "c" | "class" | "cpp" | "cs" | "h" | "java" | "php" | "py" | "sh" | "vb" | "rs" => {
                    "fa-file-code"
                }
                "rtf" | "txt" => "fa-file-alt",
                "pdf" => "fa-file-pdf",
                "doc" | "docx" => "fa-file-word",
                "7z" | "rar" | "zip" | "pkg" => "fa-file-archive",
                "m4a" | "mp3" | "ogg" | "wav" | "wma" | "webm" => "fa-file-audio",
                "avi" | "h264" | "mp4" | "mpg" | "mpeg" | "wmv" | "swf" => "fa-file-video",
                "xls" | "xlsm" | "xlsx" => "fa-file-excel",
                "pps" | "ppt" | "pptx" => "fa-file-powerpoint",
                "exe" => "fa-object-group",
                "lnk" => "fa-share-square",
                _ => "fa-file",
            }
        } else {
            "fa-file"
        }
    }

    fn compute_optimal_size(size: u64) -> String {
        // EXPERIMENTAL
        // issue #37854 <https://github.com/rust-lang/rust/issues/37854>
        /*match size {
            u64::MIN..1025 => format!("{} B", size),
            1025..1048577 => format!("{} KB", size/1024),
            1048577..1073741825 => format!("{} MB", size/1048576),
            1073741825..u64::MAX => format!("{} GB", size/1073741824),
        }*/

        let div = |size: u64, other: u64| size as f64 / other as f64;

        if size < 1025 {
            format!("{:.2} B", size)
        } else if size < 1048577 {
            format!("{:.2} KB", div(size, 1024))
        } else if size < 1073741825 {
            format!("{:.2} MB", div(size, 1048576))
        } else {
            format!("{:.2} GB", div(size, 1073741824))
        }
    }
}

impl From<Vec<Listing>> for ListingHTML {
    fn from(listings: Vec<Listing>) -> Self {
        let mut data = String::new();

        for listing in listings {
            // todo: reorganize
            let new_item = match listing {
                Listing::File { name, size } => FILE_ITEM
                    .replace("{{name}}", &name)
                    .replace("{{size}}", ListingHTML::compute_optimal_size(size).as_str())
                    .replace("{{type}}", ListingHTML::css_type_from_file_name(&name))
                    .replace("{{func}}", "download_file()"),
                Listing::Dir(name) => FILE_ITEM
                    .replace("{{name}}", &name)
                    .replace("{{type}}", "fa-folder")
                    .replace("{{size}}", "")
                    .replace("{{func}}", &*format!("change_dir('{name}')")),
                Listing::Unknown => FILE_ITEM
                    .replace("{{name}}", "unknown file")
                    .replace("{{size}}", "")
                    .replace("{{type}}", "fa-file")
                    .replace("{{func}}", "javascript:void(0)"),
            };

            data.push_str(new_item.as_str());
        }

        Self { data }
    }
}
