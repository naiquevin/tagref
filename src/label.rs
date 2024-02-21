use {
    regex::Regex,
    std::{
        fmt,
        io::BufRead,
        path::{Path, PathBuf},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Tag,
    Ref,
    File,
    Dir,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub label_type: Type,
    pub label: String,
    pub path: PathBuf,
    pub line_number: usize,
}

// Sometimes we need to be able to print a label.
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}:{}] @ {}:{}",
            match self.label_type {
                Type::Tag => "tag",
                Type::Ref => "ref",
                Type::File => "file",
                Type::Dir => "dir",
            },
            self.label,
            self.path.to_string_lossy(),
            self.line_number,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Labels {
    pub tags: Vec<Label>,
    pub refs: Vec<Label>,
    pub files: Vec<Label>,
    pub dirs: Vec<Label>,
}

// This function returns all the labels in a file for a given type.
pub fn parse<R: BufRead>(
    tag_regex: &Regex,
    ref_regex: &Regex,
    file_regex: &Regex,
    dir_regex: &Regex,
    path: &Path,
    reader: R,
) -> Labels {
    let mut tags: Vec<Label> = Vec::new();
    let mut refs: Vec<Label> = Vec::new();
    let mut files: Vec<Label> = Vec::new();
    let mut dirs: Vec<Label> = Vec::new();

    for (line_number, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            // Tags
            for captures in tag_regex.captures_iter(&line) {
                // If we got a match, then `captures.get(1)` is guaranteed to return a `Some`. Hence
                // we are justified in unwrapping.
                tags.push(Label {
                    label_type: Type::Tag,
                    label: captures.get(1).unwrap().as_str().to_owned(),
                    path: path.to_owned(),
                    line_number: line_number + 1,
                });
            }

            // Refs
            for captures in ref_regex.captures_iter(&line) {
                // If we got a match, then `captures.get(1)` is guaranteed to return a `Some`. Hence
                // we are justified in unwrapping.
                refs.push(Label {
                    label_type: Type::Ref,
                    label: captures.get(1).unwrap().as_str().to_owned(),
                    path: path.to_owned(),
                    line_number: line_number + 1,
                });
            }

            // Files
            for captures in file_regex.captures_iter(&line) {
                // If we got a match, then `captures.get(1)` is guaranteed to return a `Some`. Hence
                // we are justified in unwrapping.
                files.push(Label {
                    label_type: Type::File,
                    label: captures.get(1).unwrap().as_str().to_owned(),
                    path: path.to_owned(),
                    line_number: line_number + 1,
                });
            }

            // Directories
            for captures in dir_regex.captures_iter(&line) {
                // If we got a match, then `captures.get(1)` is guaranteed to return a `Some`. Hence
                // we are justified in unwrapping.
                dirs.push(Label {
                    label_type: Type::Dir,
                    label: captures.get(1).unwrap().as_str().to_owned(),
                    path: path.to_owned(),
                    line_number: line_number + 1,
                });
            }
        }
    }

    Labels {
        tags,
        refs,
        files,
        dirs,
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::label::{parse, Type},
        regex::Regex,
        std::path::Path,
    };

    const TAG_REGEX: &str = "(?i)\\[\\s*tag\\s*:\\s*([^\\]\\s]*)\\s*\\]";
    const REF_REGEX: &str = "(?i)\\[\\s*ref\\s*:\\s*([^\\]\\s]*)\\s*\\]";
    const FILE_REGEX: &str = "(?i)\\[\\s*file\\s*:\\s*([^\\]\\s]*)\\s*\\]";
    const DIR_REGEX: &str = "(?i)\\[\\s*dir\\s*:\\s*([^\\]\\s]*)\\s*\\]";

    #[test]
    fn parse_empty() {
        let path = Path::new("file.rs").to_owned();
        let contents = b"" as &[u8];

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents,
        );

        assert!(labels.tags.is_empty());
        assert!(labels.refs.is_empty());
        assert!(labels.files.is_empty());
        assert!(labels.dirs.is_empty());
    }

    #[test]
    fn parse_tag_basic() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?tag:label]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert_eq!(labels.tags.len(), 1);
        assert_eq!(labels.tags[0].label_type, Type::Tag);
        assert_eq!(labels.tags[0].label, "label");
        assert_eq!(labels.tags[0].path, path);
        assert_eq!(labels.tags[0].line_number, 1);
        assert!(labels.refs.is_empty());
        assert!(labels.files.is_empty());
        assert!(labels.dirs.is_empty());
    }

    #[test]
    fn parse_ref_basic() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?ref:label]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert!(labels.tags.is_empty());
        assert_eq!(labels.refs.len(), 1);
        assert_eq!(labels.refs[0].label_type, Type::Ref);
        assert_eq!(labels.refs[0].label, "label");
        assert_eq!(labels.refs[0].path, path);
        assert_eq!(labels.refs[0].line_number, 1);
        assert!(labels.files.is_empty());
        assert!(labels.dirs.is_empty());
    }

    #[test]
    fn parse_file_basic() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?file:foo/bar/baz.txt]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert!(labels.tags.is_empty());
        assert!(labels.refs.is_empty());
        assert_eq!(labels.files.len(), 1);
        assert_eq!(labels.files[0].label_type, Type::File);
        assert_eq!(labels.files[0].label, "foo/bar/baz.txt");
        assert_eq!(labels.files[0].path, path);
        assert_eq!(labels.files[0].line_number, 1);
        assert!(labels.dirs.is_empty());
    }

    #[test]
    fn parse_dir_basic() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?dir:foo/bar/baz]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert!(labels.tags.is_empty());
        assert!(labels.refs.is_empty());
        assert!(labels.files.is_empty());
        assert_eq!(labels.dirs.len(), 1);
        assert_eq!(labels.dirs[0].label_type, Type::Dir);
        assert_eq!(labels.dirs[0].label, "foo/bar/baz");
        assert_eq!(labels.dirs[0].path, path);
        assert_eq!(labels.dirs[0].line_number, 1);
    }

    #[test]
    fn parse_multiple_per_line() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?tag:label][?ref:label][?file:foo/bar/baz.txt][?dir:foo/bar/baz]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert_eq!(labels.tags.len(), 1);
        assert_eq!(labels.tags[0].label_type, Type::Tag);
        assert_eq!(labels.tags[0].label, "label");
        assert_eq!(labels.tags[0].path, path);
        assert_eq!(labels.tags[0].line_number, 1);

        assert_eq!(labels.refs.len(), 1);
        assert_eq!(labels.refs[0].label_type, Type::Ref);
        assert_eq!(labels.refs[0].label, "label");
        assert_eq!(labels.refs[0].path, path);
        assert_eq!(labels.refs[0].line_number, 1);

        assert_eq!(labels.files.len(), 1);
        assert_eq!(labels.files[0].label_type, Type::File);
        assert_eq!(labels.files[0].label, "foo/bar/baz.txt");
        assert_eq!(labels.files[0].path, path);
        assert_eq!(labels.files[0].line_number, 1);

        assert_eq!(labels.dirs.len(), 1);
        assert_eq!(labels.dirs[0].label_type, Type::Dir);
        assert_eq!(labels.dirs[0].label, "foo/bar/baz");
        assert_eq!(labels.dirs[0].path, path);
        assert_eq!(labels.dirs[0].line_number, 1);
    }

    #[test]
    fn parse_multiple_lines() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?tag:label]
      [?ref:label]
      [?file:foo/bar/baz.txt]
      [?dir:foo/bar/baz]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert_eq!(labels.tags.len(), 1);
        assert_eq!(labels.tags[0].label_type, Type::Tag);
        assert_eq!(labels.tags[0].label, "label");
        assert_eq!(labels.tags[0].path, path);
        assert_eq!(labels.tags[0].line_number, 1);

        assert_eq!(labels.refs.len(), 1);
        assert_eq!(labels.refs[0].label_type, Type::Ref);
        assert_eq!(labels.refs[0].label, "label");
        assert_eq!(labels.refs[0].path, path);
        assert_eq!(labels.refs[0].line_number, 2);

        assert_eq!(labels.files.len(), 1);
        assert_eq!(labels.files[0].label_type, Type::File);
        assert_eq!(labels.files[0].label, "foo/bar/baz.txt");
        assert_eq!(labels.files[0].path, path);
        assert_eq!(labels.files[0].line_number, 3);

        assert_eq!(labels.dirs.len(), 1);
        assert_eq!(labels.dirs[0].label_type, Type::Dir);
        assert_eq!(labels.dirs[0].label, "foo/bar/baz");
        assert_eq!(labels.dirs[0].path, path);
        assert_eq!(labels.dirs[0].line_number, 4);
    }

    #[test]
    fn parse_whitespace() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [  ?tag   :  label            ]
      [  ?ref   :  label            ]
      [  ?file  :  foo/bar/baz.txt  ]
      [  ?dir   :  foo/bar/baz      ]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert_eq!(labels.tags.len(), 1);
        assert_eq!(labels.tags[0].label_type, Type::Tag);
        assert_eq!(labels.tags[0].label, "label");
        assert_eq!(labels.tags[0].path, path);
        assert_eq!(labels.tags[0].line_number, 1);

        assert_eq!(labels.refs.len(), 1);
        assert_eq!(labels.refs[0].label_type, Type::Ref);
        assert_eq!(labels.refs[0].label, "label");
        assert_eq!(labels.refs[0].path, path);
        assert_eq!(labels.refs[0].line_number, 2);

        assert_eq!(labels.files.len(), 1);
        assert_eq!(labels.files[0].label_type, Type::File);
        assert_eq!(labels.files[0].label, "foo/bar/baz.txt");
        assert_eq!(labels.files[0].path, path);
        assert_eq!(labels.files[0].line_number, 3);

        assert_eq!(labels.dirs.len(), 1);
        assert_eq!(labels.dirs[0].label_type, Type::Dir);
        assert_eq!(labels.dirs[0].label, "foo/bar/baz");
        assert_eq!(labels.dirs[0].path, path);
        assert_eq!(labels.dirs[0].line_number, 4);
    }

    #[test]
    fn parse_case() {
        let path = Path::new("file.rs").to_owned();
        let contents = r"
      [?tag:label]
      [?TAG:LABEL]
      [?ref:label]
      [?REF:LABEL]
      [?file:foo/bar/baz.txt]
      [?FILE:FOO/BAR/BAZ.TXT]
      [?dir:foo/bar/baz]
      [?DIR:FOO/BAR/BAZ]
    "
        .trim()
        .replace('?', "")
        .as_bytes()
        .to_owned();

        let tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
        let ref_regex: Regex = Regex::new(REF_REGEX).unwrap();
        let file_regex: Regex = Regex::new(FILE_REGEX).unwrap();
        let dir_regex: Regex = Regex::new(DIR_REGEX).unwrap();

        let labels = parse(
            &tag_regex,
            &ref_regex,
            &file_regex,
            &dir_regex,
            &path,
            contents.as_ref(),
        );

        assert_eq!(labels.tags.len(), 2);
        assert_eq!(labels.tags[0].label_type, Type::Tag);
        assert_eq!(labels.tags[0].label, "label");
        assert_eq!(labels.tags[0].path, path);
        assert_eq!(labels.tags[0].line_number, 1);
        assert_eq!(labels.tags[1].label_type, Type::Tag);
        assert_eq!(labels.tags[1].label, "LABEL");
        assert_eq!(labels.tags[1].path, path);
        assert_eq!(labels.tags[1].line_number, 2);

        assert_eq!(labels.refs.len(), 2);
        assert_eq!(labels.refs[0].label_type, Type::Ref);
        assert_eq!(labels.refs[0].label, "label");
        assert_eq!(labels.refs[0].path, path);
        assert_eq!(labels.refs[0].line_number, 3);
        assert_eq!(labels.refs[1].label_type, Type::Ref);
        assert_eq!(labels.refs[1].label, "LABEL");
        assert_eq!(labels.refs[1].path, path);
        assert_eq!(labels.refs[1].line_number, 4);

        assert_eq!(labels.files.len(), 2);
        assert_eq!(labels.files[0].label_type, Type::File);
        assert_eq!(labels.files[0].label, "foo/bar/baz.txt");
        assert_eq!(labels.files[0].path, path);
        assert_eq!(labels.files[0].line_number, 5);
        assert_eq!(labels.files[1].label_type, Type::File);
        assert_eq!(labels.files[1].label, "FOO/BAR/BAZ.TXT");
        assert_eq!(labels.files[1].path, path);
        assert_eq!(labels.files[1].line_number, 6);

        assert_eq!(labels.dirs.len(), 2);
        assert_eq!(labels.dirs[0].label_type, Type::Dir);
        assert_eq!(labels.dirs[0].label, "foo/bar/baz");
        assert_eq!(labels.dirs[0].path, path);
        assert_eq!(labels.dirs[0].line_number, 7);
        assert_eq!(labels.dirs[1].label_type, Type::Dir);
        assert_eq!(labels.dirs[1].label, "FOO/BAR/BAZ");
        assert_eq!(labels.dirs[1].path, path);
        assert_eq!(labels.dirs[1].line_number, 8);
    }
}
