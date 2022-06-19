/// Tries to match most popular extensions with proper
/// [mime type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types)
///
/// Returns "`application/octet-stream`" if no match was found
pub fn extension_to_mime(ext: impl AsRef<str>) -> &'static str {
    match ext.as_ref() {
        "avi" => "video/x-msvideo",
        "bmp" => "image/bmp",
        "css" => "text/css",
        "csv" => "text/csv",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "gz" => "application/gzip",
        "gif" => "image/gif",
        "htm" | "html" => "text/html",
        "ico" => "image/vnd.microsoft.icon",
        "jar" => "application/java-archive",
        "jpeg" | "jpg" => "image/jpeg",
        "js" => "text/javascript",
        "json" => "application/json",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "mpeg" => "video/mpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "php" => "application/x-httpd-php",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "rar" => "application/vnd.rar",
        "rtf" => "application/rtf",
        "sh" => "application/x-sh",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        "wav" => "audio/wav",
        "weba" => "audio/webm",
        "webm" => "video/webm",
        "webp" => "image/webp",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "7z" => "application/x-7z-compressed",
        &_ => "application/octet-stream",
    }
}
