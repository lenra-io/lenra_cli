use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use regex::Regex;
use tokio::process::Command;

pub fn pull(image: String) -> Command {
    let mut cmd = Command::new("docker");
    cmd.kill_on_drop(true).arg("pull").arg(image);
    cmd
}

pub fn normalize_tag(tag: String) -> String {
    let re = Regex::new(r"[^A-Za-z0-9._-]").unwrap();
    let tag = re.replace_all(tag.as_str(), "-").to_string();
    if tag.len() > 63 {
        let mut hacher = DefaultHasher::new();
        tag.hash(&mut hacher);
        let hash = format!("{:X}", hacher.finish());
        format!(
            "{}{}",
            tag.chars().take(63 - hash.len()).collect::<String>(),
            hash
        )
    } else {
        tag
    }
}

#[cfg(test)]
mod test_normalize_tag {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use regex::Regex;

    use super::normalize_tag;

    #[test]
    fn prefixed_tag_name() {
        let tag_name = "prefixed/branch-name".to_string();
        assert_eq!(normalize_tag(tag_name), "prefixed-branch-name".to_string());
    }

    #[test]
    fn long_tag_name() {
        let tag_name =
            "prefixed/branch-name/with_many-many.many_many&many-many/underscore".to_string();
        let re = Regex::new(r"[^A-Za-z0-9._-]").unwrap();
        let tag = re.replace_all(tag_name.as_str(), "-").to_string();
        let mut hacher = DefaultHasher::new();
        tag.hash(&mut hacher);
        let hash = format!("{:X}", hacher.finish());
        let tag = format!(
            "{}{}",
            tag.chars().take(63 - hash.len()).collect::<String>(),
            hash
        );
        assert_eq!(normalize_tag(tag_name), tag.to_string());
    }
}
