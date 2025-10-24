use std::{
    env,
    fs::{self, File},
    io::Write,
};

const COMMANDS: &[&str] = &[
    "rink_query",
    "symbols_query",
    "applications_query",
    "start_application",
    "resolve_icon",
    "reload_desktop_files",
];

fn escape_unicode_control_chars(input: &str) -> String {
    use regex::Regex;

    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"[\u{202a}-\u{202e}\u{2066}-\u{2069}]").unwrap());

    re.replace_all(input, |caps: &regex::Captures| {
        let c = caps.get(0).unwrap().as_str().chars().next().unwrap();
        format!("\\u{{{:04x}}}", c as u32)
    }).into_owned()
}

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();

    // Never rerun unless build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // See https://github.com/anyrun-org/anyrun/blob/master/plugins/symbols/build.rs
    let string =
        fs::read_to_string("./data/unicodeData.txt").expect("Failed to load unicode data!");
    let mut file = File::create(format!("{}/unicode.rs", env::var("OUT_DIR").unwrap()))
        .expect("Unable to create unicode output file!");

    file.write_all(b"const UNICODE_CHARS: &[(&str, &str)] = &[\n").unwrap();
    string.lines().for_each(|line| {
        let fields = line.split(';').collect::<Vec<_>>();
        let chr = match char::from_u32(u32::from_str_radix(fields[0], 16).unwrap()) {
            Some(char) => char,
            None => return,
        };

        if fields[1] != "<control>" {
            let chr_str = &chr.to_string();
            file.write_all(format!("(r#\"{}\"#, r#\"{}\"#),\n", fields[1], escape_unicode_control_chars(chr_str)).as_bytes())
                .unwrap();
        }
    });

    file.write_all(b"];\n").unwrap();
}
