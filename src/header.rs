use chrono::Local;
use std::collections::HashMap;
use std::env::var;
use std::sync::LazyLock;

static ART: [&str; 7] = [
    "        :::      ::::::::",
    "      :+:      :+:    :+:",
    "    +:+ +:+         +:+  ",
    "  +#+  +:+       +#+     ",
    "+#+#+#+#+#+   +#+        ",
    "     #+#    #+#          ",
    "    ###   ########.fr    ",
];

#[allow(clippy::single_element_loop)]
static TYPES: LazyLock<HashMap<&'static str, [&'static str; 3]>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for t in [".c", ".cpp", ".h", ".hpp", ".php"] {
        m.insert(t, ["/*", "*/", "*"]);
    }
    for t in [".htm", ".html", ".xml"] {
        m.insert(t, ["<!--", "-->", "*"]);
    }
    for t in [".js"] {
        m.insert(t, ["//", "//", "*"]);
    }
    for t in [".tex"] {
        m.insert(t, ["%", "%", "*"]);
    }
    for t in [".ml", ".mli", ".mll", ".mly"] {
        m.insert(t, ["(*", "*)", "*"]);
    }
    for t in [".vim", "vimrc"] {
        m.insert(t, ["\"", "\"", "*"]);
    }
    for t in [".el", "emacs"] {
        m.insert(t, [";", ";", "*"]);
    }
    for t in [".f90", ".f95", ".f03", ".f", ".for"] {
        m.insert(t, ["!", "!", "/"]);
    }

    m
});

fn make_top_bottom_lines(header: &mut [String; 11], start: &str, end: &str, fill: &str) {
    let end_idx = header.len() - 1;
    let mut fill_row = |row: usize| {
        header[row].push_str(start);
        header[row].push(' ');
        for _ in 0..(LEN - start.len() - end.len() - 2) {
            header[row].push_str(fill);
        }
        header[row].push(' ');
        header[row].push_str(end);
    };
    fill_row(0);
    fill_row(end_idx);
}

// return s:start
// . repeat(' ', s:margin - strlen(s:start))
// . l:left
// . repeat(' ',
//          s:length - s:margin * 2 - strlen(l:left) - strlen(a:right))
// . a:right
// . repeat(' ', s:margin - strlen(s:end))
// . s:end

fn text_line(line: &mut String, left: &str, right: &str, (start, end): (&str, &str)) {
    let l = &left[0..(left
        .char_indices()
        .nth(LEN - MARGIN * 2 - right.len())
        .map(|(i, _)| i)
        .unwrap_or(left.as_bytes().len()))];
    line.push_str(start);
    for _ in 0..(MARGIN - start.len()) {
        line.push(' ');
    }
    line.push_str(l);
    for _ in 0..(LEN - MARGIN * 2 - l.len() - right.len()) {
        line.push(' ');
    }
    line.push_str(right);
    for _ in 0..(MARGIN - end.len()) {
        line.push(' ');
    }
    line.push_str(end);
}

const LEN: usize = 80;
const MARGIN: usize = 5;

#[allow(dead_code, unused)]
pub fn insert_header(
    filename: &str,
    output: &mut impl std::io::Write,
    current_header: Option<[String; 11]>,
) -> std::io::Result<()> {
    let user = var("USER").ok().unwrap_or_else(|| "marvin".to_string());
    let mail = var("MAIL")
        .ok()
        .unwrap_or_else(|| "marvin@42.fr".to_string());
    let time = Local::now().format("%Y/%m/%d %H:%M:%S");
    let [s, e, m] = TYPES
        .iter()
        .filter(|&(k, _)| filename.ends_with(k))
        .map(|(_, v)| *v)
        .next()
        .unwrap_or(["#", "#", "*"]);
    let mut header = current_header.unwrap_or_else(|| {
        let mut out: [String; 11] = std::array::from_fn(|_| String::with_capacity(LEN + 1));
        make_top_bottom_lines(&mut out, s, e, m);

        // BLANK LINE
        for i in [1, 9] {
            text_line(&mut out[i], "", "", (s, e));
        }

        // BLANK + ASCII
        for i in [2, 4, 6] {
            text_line(&mut out[i], "", ART[i - 2], (s, e));
        }

        // FILENAME
        {
            let i = 3;
            text_line(&mut out[i], filename, ART[i - 2], (s, e));
        }
        // AUTHOR
        {
            let i = 5;
            text_line(
                &mut out[i],
                &format!("By: {user} <{mail}>"),
                ART[i - 2],
                (s, e),
            );
        }

        // CREATED AT
        {
            let i = 7;
            text_line(
                &mut out[i],
                &format!("Created: {time} by {user}"),
                ART[i - 2],
                (s, e),
            );
        }
        // UPDATED AT
        {
            let i = 8;
            text_line(
                &mut out[i],
                &format!("Updated: {time} by {user}"),
                ART[i - 2],
                (s, e),
            );
        }

        out
    });
    // UPDATED AT
    {
        let i = 8;
        header[i].clear();
        text_line(
            &mut header[i],
            &format!("Updated: {time} by {user}"),
            ART[i - 2],
            (s, e),
        );
    }

    for line in header {
        writeln!(output, "{}", line)?;
    }

    Ok(())
}
