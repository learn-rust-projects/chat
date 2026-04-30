use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};

use crate::models::ChatFile;

impl ChatFile {
    pub fn new(filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ext: filename
                .rsplit_once('.')
                .map(|(_, ext)| ext)
                .unwrap_or("txt")
                .to_string(),
            hash: hex::encode(hash),
        }
    }
    pub fn url(&self, ws_id: i64) -> String {
        format!("/files/{ws_id}/{}", self.hash_to_path())
    }
    pub fn path(&self, ws_id: i64, base_dir: &Path) -> PathBuf {
        base_dir.join(ws_id.to_string()).join(self.hash_to_path())
    }

    // split hash into 3 parts, first 2 with 3 chars
    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new("test.txt", b"hello world");
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
    #[test]
    fn chat_file_hash_to_path_should_work() {
        let file = ChatFile::new("test.txt", b"hello world");
        assert_eq!(
            file.path(123, Path::new("/files")),
            PathBuf::from("/files/123/2aa/e6c/35c94fcfb415dbe95f408b9ce91ee846ed.txt")
        );
    }
    #[test]
    fn chat_file_url_should_work() {
        let file = ChatFile::new("test.txt", b"hello world");
        assert_eq!(
            file.url(123),
            "/files/123/2aa/e6c/35c94fcfb415dbe95f408b9ce91ee846ed.txt"
        );
    }
}
