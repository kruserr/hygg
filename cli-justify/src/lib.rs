fn split_at_char(s: &str, n: usize) -> (&str, Option<&str>) {
  for (char_index, (i, _)) in s.char_indices().enumerate() {
    if char_index == n {
      let (w1, w2) = s.split_at(i);
      return (w1, Some(w2));
    }
  }

  (s, None)
}

pub fn justify(text: &str, line_width: usize) -> Vec<String> {
  let paragraphs: Vec<&str> = text.split("\n\n").collect();
  let mut lines: Vec<String> = Vec::new();

  for paragraph in paragraphs {
    let raw_words: Vec<&str> = paragraph.split_whitespace().collect();
    let mut words = vec![];

    for mut word in raw_words {
      while let (w1, Some(w2)) = split_at_char(word, line_width) {
        words.push(w1);
        word = w2;
      }

      words.push(word);
    }

    let mut line: Vec<&str> = Vec::new();
    let mut len = 0;

    for word in words {
      // Calculate the length if we add this word
      let word_len = word.len();
      let space_len = if line.is_empty() { 0 } else { 1 };
      let new_len = len + space_len + word_len;

      // If adding this word would exceed the line width and we have words on
      // the line
      if new_len > line_width && !line.is_empty() {
        lines.push(justify_line(&line, line_width));
        line.clear();
        len = 0;
      }

      line.push(word);
      len = if line.len() == 1 { word_len } else { len + space_len + word_len };
    }

    // Add the last line of the paragraph
    if !line.is_empty() {
      lines.push(line.join(" "));
    }

    // Add a blank line after each paragraph to preserve paragraph breaks
    lines.push(String::new());
  }

  lines
}

fn justify_line(line: &[&str], line_width: usize) -> String {
  let word_len: usize = line.iter().map(|s| s.len()).sum();

  // If the words are already longer than or equal to line width,
  // or if there's only one word, just join them with single spaces
  if word_len >= line_width || line.len() <= 1 {
    return line.join(" ");
  }

  let spaces = line_width - word_len;

  let line_len_div = if (line.len() > 1) { (line.len() - 1) } else { 1 };

  let each_space = spaces / line_len_div;
  let extra_space = spaces % line_len_div;

  let mut justified = String::new();
  for (i, word) in line.iter().enumerate() {
    justified.push_str(word);
    if i < line.len() - 1 {
      let mut space = " ".repeat(each_space);
      if i < extra_space {
        space.push(' ');
      }
      justified.push_str(&space);
    }
  }

  justified
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_handles_long_words() {
    let input_text = r#"some text and a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong word but no cause to panic"#;
    let pretty_short_line_width = 10;
    let result = justify(input_text, pretty_short_line_width);
    assert!(!result.is_empty());
  }

  #[test]
  fn test_handles_line_longer_than_width() {
    let input_text =
      "This is a line that is definitely longer than the requested width";
    let result = justify(input_text, 20);
    assert!(!result.is_empty());
    // Should not panic
  }

  #[test]
  fn test_single_word_longer_than_width() {
    let input_text = "supercalifragilisticexpialidocious";
    let result = justify(input_text, 10);
    assert!(!result.is_empty());
    // Word should be split into multiple lines
    assert!(result.len() > 1);
  }

  #[test]
  fn test_normal_justification() {
    // Test with multiple lines to see justification
    let input_text = "This is a test of the justification system. It should properly justify lines that need to be wrapped.";
    let result = justify(input_text, 20);
    assert!(!result.is_empty());

    // Find a line that was justified (not the last line)
    let mut found_justified = false;
    for (i, line) in result.iter().enumerate() {
      if !line.is_empty() && i < result.len() - 2 {
        // Not the last line or blank line
        if line.len() == 20 {
          found_justified = true;
          break;
        }
      }
    }
    assert!(found_justified, "Should have at least one justified line");
  }
}
