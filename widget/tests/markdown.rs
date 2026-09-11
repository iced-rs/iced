//! Tests for the incremental [`iced_widget::markdown`] parser: the
//! [`Content::push_str`] method must converge to the one-shot
//! [`Content::parse`] result, whatever the chunks are.
//!
//! [`Content::push_str`]: iced_widget::markdown::Content::push_str
//! [`Content::parse`]: iced_widget::markdown::Content::parse

use iced_widget::markdown::Content;

/// Asserts that parsing `full` incrementally, with different chunking
/// schemes, converges to the one-shot parse.
fn assert_converges(full: &str, label: &str) {
    let mut idx = 0;
    let three = std::iter::from_fn(|| {
        if idx >= full.len() {
            return None;
        }
        let end = (idx + 3).min(full.len());
        let chunk = full[idx..end].to_owned();
        idx = end;
        Some(chunk)
    })
    .collect::<Vec<_>>();

    for (chunks, name) in [
        (
            full.split_inclusive(' ')
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            "word-by-word",
        ),
        (
            full.chars().map(|c| c.to_string()).collect::<Vec<_>>(),
            "char-by-char",
        ),
        (three, "3-char chunks"),
    ] {
        let mut c = Content::new();
        for chunk in &chunks {
            c.push_str(chunk);
        }

        let one = Content::parse(full);

        assert_eq!(
            format!("{:?}", c.items()),
            format!("{:?}", one.items()),
            "{label}: incremental ({name}) should converge to one-shot"
        );
        assert_eq!(
            c.images(),
            one.images(),
            "{label}: images should converge ({name})"
        );
    }
}

/// A long list of bullets.
#[test]
fn bullets() {
    let full = (0..50).map(|i| format!("- item {i}\n")).collect::<String>();
    assert_converges(&full, "bullets");
}

/// A long ordered list.
#[test]
fn ordered_list() {
    let full = (1..=30)
        .map(|i| format!("{i}. step {i}\n"))
        .collect::<String>();
    assert_converges(&full, "ordered list");
}

/// A task list.
#[test]
fn task_list() {
    let full = "- [ ] a\n- [x] b\n- [ ] c\n- [x] d\n- [ ] e\n";
    assert_converges(full, "task list");
}

/// A nested list.
#[test]
fn nested_list() {
    let full = "- a\n  - b\n  - c\n- d\n  - e\n- f\n";
    assert_converges(full, "nested list");
}

/// A loose list (blank lines between bullets).
#[test]
fn loose_list() {
    let full = "- a\n\n- b\n\n- c\n";
    assert_converges(full, "loose list");
}

/// Bullets with multiple paragraphs.
#[test]
fn multi_paragraph_bullets() {
    let full = "- a\n\n  more a\n\n- b\n\n  more b\n";
    assert_converges(full, "multi paragraph bullets");
}

/// A list surrounded by other items.
#[test]
fn list_surrounded_by_other_items() {
    let full = "# Title\n\nintro\n\n- a\n- b\n\nafter\n\n- c\n";
    assert_converges(full, "list surrounded by other items");
}

/// A code block followed by a list.
#[test]
fn code_block_and_list() {
    let full = "```\ncode\n```\n\n- a\n- b\n";
    assert_converges(full, "code block and list");
}

/// A list inside a quote, followed by another list.
#[test]
fn list_in_quote() {
    let full = "> - a\n> - b\n\n- c\n";
    assert_converges(full, "list in quote");
}

/// A bullet that looks like a setext heading underline.
#[test]
fn setext_like_bullet() {
    let full = "- a\n  ---\n- b\n";
    assert_converges(full, "setext like bullet");
}

/// A bullet containing a link and an image.
#[test]
fn bullet_with_link_and_image() {
    let full = "- [link](https://a.com) ![img](https://b.com/i.png)\n- b\n";
    assert_converges(full, "bullet with link and image");
}

/// A bullet ending with a pipe (table-ish).
#[test]
fn bullet_ending_with_pipe() {
    let full = "- foo |\n- bar\n";
    assert_converges(full, "bullet ending with pipe");
}

/// A single very long bullet.
#[test]
fn long_single_bullet() {
    let full = format!("- {}\n", "word ".repeat(500).trim_end());
    assert_converges(&full, "long single bullet");
}

/// A broken reference link, resolved once the reference (defined
/// after the list) arrives.
#[test]
fn broken_reference_link_resolved_later() {
    let full = "- a [x][ref]\n- b [y][ref]\n- c\n\n[ref]: https://example.com\n";
    assert_converges(full, "broken reference link resolved later");
}

/// A broken reference link, resolved between two bullets of the same
/// list.
#[test]
fn broken_reference_link_resolved_between_bullets() {
    let full = "- a [x][ref]\n\n[ref]: https://example.com\n\n- b [y][ref]\n";
    assert_converges(full, "broken reference link resolved between bullets");
}

/// A table, streamed word by word.
#[test]
fn table_word_by_word() {
    let full = "| a | b |\n| - | - |\n| 1 | 2 |\n| 3 | 4 |\n\nDone.\n";
    let mut c = Content::new();
    for w in full.split_inclusive(' ') {
        c.push_str(w);
    }
    let one = Content::parse(full);
    assert_eq!(
        format!("{:?}", c.items()),
        format!("{:?}", one.items()),
        "incremental should converge to one-shot"
    );
}

/// An image and a reference link on the same line; the image must not
/// be duplicated as the line grows.
#[test]
fn image_and_link_on_same_line() {
    let full = "prev\n\n![alt](https://img.com/i.png) [x][ref]\n\ntext\n";
    let mut c = Content::new();
    for w in full.split_inclusive('\n') {
        c.push_str(w);
    }
    let one = Content::parse(full);
    assert_eq!(
        format!("{:?}", c.items()),
        format!("{:?}", one.items()),
        "incremental should converge to one-shot"
    );
}

/// Several images and a paragraph on the same line; the images and
/// the paragraph share the same source region, and the images must
/// not be duplicated as the line grows.
#[test]
fn images_on_same_line() {
    let full = "a\n\n![i1](https://u1.com/1.png) ![i2](https://u2.com/2.png) t\n\nb\n";
    assert_converges(full, "images on the same line");
}

/// A line with only images, followed by more content.
#[test]
fn image_only_line() {
    let full = "a\n\n![i1](https://u1.com/1.png) ![i2](https://u2.com/2.png)\n\nb\n";
    assert_converges(full, "image-only line");
}

/// Text followed by an image on the same line; the text must not be
/// absorbed into the image's alt.
#[test]
fn text_then_image() {
    let full = "hello ![img](https://u1.com/1.png)\n\nmore\n";
    assert_converges(full, "text then image");
}

/// A reference defined before it is used.
#[test]
fn reference_before_use() {
    let full = "[ref]: https://example.com\n\n- [x][ref]\n- [y][ref]\n";
    let mut c = Content::new();
    for chunk in full.chars().map(|c| c.to_string()) {
        c.push_str(&chunk);
    }
    let one = Content::parse(full);
    assert_eq!(
        format!("{:?}", c.items()),
        format!("{:?}", one.items()),
        "char-by-char should converge to one-shot"
    );
}

/// A metadata block after content should not break the flow.
///
/// Note: a metadata block at the very start of the stream cannot
/// converge, as the opening delimiter (`---` or `+++`) is committed
/// as a `Rule`/paragraph before the block can be determined.
#[test]
fn metadata_after_content() {
    let full = "- a\n\n+++\nmeta: 1\n+++\n\n- b\n";
    let mut c = Content::new();
    c.push_str("");
    for chunk in full.chars().map(|c| c.to_string()) {
        c.push_str(&chunk);
    }
    let one = Content::parse(full);
    assert_eq!(
        format!("{:?}", c.items()),
        format!("{:?}", one.items()),
        "char-by-char should converge to one-shot"
    );
}

/// Many constructs at once, streamed block by block, word by word and
/// char by char.
#[test]
fn mixed_document() {
    let parts = [
        "# Title\n\n",
        "intro with **bold** and *emphasis* and `code`\n\n",
        "- a\n",
        "- b [link](https://a.com)\n",
        "  - nested\n",
        "- [ ] task\n",
        "- [x] done\n\n",
        "1. one\n",
        "2. two\n\n",
        "> quote\n",
        "> - in a quote\n\n",
        "```\ncode block\n```\n\n",
        "![alt](https://img.com/i.png) [x][ref]\n\n",
        "text\n\n",
        "[ref]: https://example.com\n\n",
        "- c\n",
        "- d\n\n",
        "| a | b |\n| - | - |\n| 1 | 2 |\n",
        "\n\nafter\n",
    ];
    let full = parts.concat();

    // Whole-document chunks, word chunks, char chunks
    for (chunks, name) in [
        (
            parts.iter().map(ToString::to_string).collect(),
            "block-by-block",
        ),
        (
            full.split_inclusive(' ')
                .map(str::to_owned)
                .collect::<Vec<_>>(),
            "word-by-word",
        ),
        (
            full.chars().map(|c| c.to_string()).collect::<Vec<_>>(),
            "char-by-char",
        ),
    ] {
        let mut c = Content::new();
        for chunk in &chunks {
            c.push_str(chunk);
        }
        let one = Content::parse(&full);
        assert_eq!(
            format!("{:?}", c.items()),
            format!("{:?}", one.items()),
            "{name} should converge to one-shot"
        );
        assert_eq!(c.images(), one.images(), "images ({name})");
    }
}

/// Pushing new bullets to a long list must stay cheap: `push_str`
/// re-parses only the last bullet, so the time per push does not grow
/// with the list.
#[test]
fn pushing_to_a_long_list_stays_fast() {
    let mut c = Content::new();
    let bullet = |i| format!("- item {i} with some **text** and a `code`\n");

    for i in 0..800 {
        c.push_str(&bullet(i));
    }

    // The last 200 pushes, as a fraction of the total, must be cheap:
    // with an O(n) re-parse per push they would take most of the time.
    let start = std::time::Instant::now();
    for i in 800..1000 {
        c.push_str(&bullet(i));
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 2.0,
        "the last 200 pushes took {elapsed:?}, which suggests that the \
         whole list is re-parsed on every push"
    );
    assert_eq!(c.items().len(), 1);
}

#[test]
fn reference_resolved_before_next_item() {
    let full = "- a [x][ref]\n\n[ref]: https://example.com\n\npara\n";
    assert_converges(full, "reference resolved before the next item");
}

#[test]
fn duplicate_reference_definitions() {
    // The first definition wins, like in CommonMark; the second
    // definition is still growing (not terminated) when the push
    // happens.
    let full = "- [x][ref]\n\n[ref]: https://one.com\n\npara\n\n[ref]: https://two.com\n";
    assert_converges(full, "duplicate reference definitions");
}

#[test]
fn broken_link_in_image_alt() {
    let full = "before\n\n![alt with [x][ref]](https://img.com/i.png)\n\n[ref]: https://example.com\n\nafter\n";
    assert_converges(full, "broken link in an image alt");
}

#[test]
fn broken_link_in_image_alt_with_trailing_text() {
    // The section is created for the image before its paragraph is
    // complete; the re-parse range must then span the whole line.
    let full =
        "![alt [x][ref]](https://img.com/i.png) trailing\n\n[ref]: https://example.com\n\nafter\n";
    assert_converges(full, "broken link in an image alt, with trailing text");
}

/// A rule directly after a list item, with no blank line in between:
/// the rule ends the list, and the incremental parse must keep both
/// the list and the rule.
#[test]
fn list_then_rule_without_blank_line() {
    let full = "[other]: https://other.com\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n\
         [ref]: https://example.com\n\n- a\n---\n\n";
    assert_converges(full, "list then rule without blank line");
}

/// A reference definition directly after a list line, with no blank
/// line in between, is absorbed into the bullet (lazy continuation);
/// the list that follows must not be lost.
#[test]
fn reference_then_list_without_blank_line() {
    let full =
        "2. two\n[ref]: https://changed.com\n\n\n- a\n| a | b |\n| - | - |\n| 1 | 2 |\n\n---\n\n";
    assert_converges(full, "reference then list without blank line");
}

/// A reference definition absorbed into a quoted bullet (lazy
/// continuation) embeds a link to the label it defines; that link
/// must resolve with the one-shot semantics: the first definition of
/// the label wins, like in CommonMark.
#[test]
fn absorbed_reference_resolves_to_first_definition() {
    let full = "[ref]: https://example.com\n\n> - in quote\n[ref]: https://changed.com\n\n\
         [ref]: https://changed.com\n\n> - in quote\n> - in quote\n- b [y][ref]\n\n";
    assert_converges(full, "absorbed reference resolves to the first definition");
}
