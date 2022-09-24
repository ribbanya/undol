use regex::Regex;
use crate::{MainError::{self, *}};
// symbol_re = re.compile(
// r"^  (?P<local_addr>[\da-f]{8}) (?P<size>[\da-f]{6})"
// r" (?P<global_addr>[\da-f]{8})(?:  (?P<alignment>\d))?"
// r" (?P<name>@?[\.\w]+)(?: \(entry of (?P<parent>@?[\.\w]+)\))?"
// r" \t(?P<filename>@?[\w \.]+)\.o $")



pub(crate) fn parse_map(map_text: &str) -> Result<(), MainError> {
    let symbol_re = Regex::new(r"(?x)
            ^(?:
                (?P<section>\.(?P<section_name>\w+)\ section\ layout)
                | (?P<symbol>
                    # Offset from the start of the section
                    \ {2}(?P<local_addr>[\da-f]{8})

                    # Symbol size, up to six digits
                    \ (?P<size>[\da-f]{6})

                    # Virtual address
                    \ (?P<global_addr>[\da-f]{8})
                    
                    # Some entries have alignment
                    (?:\ (?P<alignment>[\d ]\d))?

                    # Symbol name, valid C identifier
                    \ (?P<symbol_name>@?[.\w]+)
                    
                    # Some entries have a parent (what struct or section they belong to)
                    (?:\ \(entry\ of\ (?P<parent_name>@?[.\w]+)\))?

                    # Filename of the containing object file
                    \ \t(?P<filename>@?[\w .]+)\.o\x20
                )
            )$")
        .map_err(RegexError)?;

    let mut section_name = "";
    for line in map_text.lines() {
        if let Some(captures) = symbol_re.captures(line) {
            if let Some(r#match) = captures.name("section_name") {
                section_name = r#match.as_str();
            }
        }
        println!("{}", section_name);
    }
    // for capture in symbol_re.captures_iter(&map_text) {
    //     if let Some(s) = capture.name("section").map(|m| m.as_str()) {
    //         println!("{s}")
    //     }
    // }
    Ok(())
}
