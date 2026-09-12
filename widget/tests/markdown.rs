//! Tests for the [`iced_widget::markdown`] parser: the one-shot
//! [`parse`] and [`sections`] functions, and the incremental
//! [`Content::push_str`] method, which must converge to the one-shot
//! [`Content::parse`] result, whatever the chunks are.
//!
//! [`parse`]: iced_widget::markdown::parse
//! [`sections`]: iced_widget::markdown::sections
//! [`Content::push_str`]: iced_widget::markdown::Content::push_str
//! [`Content::parse`]: iced_widget::markdown::Content::parse
use iced_widget::markdown::{Content, Item, parse, sections};

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

/// A reference whose definition is swallowed by a metadata block must
/// stop resolving the links that were resolved with it while the block
/// was still open.
#[test]
fn reference_swallowed_by_metadata_block() {
    let full = "para\n\n- [x][ref]\n---\nkey: value\n| a | b |\n| - | - |\n| 1 | 2 |\n\n2. two [y][ref]\n![alt with [x][ref]](https://img.com/i.png)\n\n+++\nkey: value\n+++\n\n[ref]: https://changed.com\n\n---";
    assert_converges(full, "reference swallowed by a metadata block");
}

/// A reference defined both inside a swallowed metadata block and
/// outside it resolves to the outside definition, like the one-shot
/// parse does.
#[test]
fn reference_first_defined_in_swallowed_block() {
    let full =
        "a [x][ref]\n\n---\n[ref]: https://in_block.com\n---\n\n[ref]: https://outside.com\n";
    assert_converges(full, "reference first defined in a swallowed block");
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
    assert_converges(full, "table word by word");
}

/// An image and a reference link on the same line; the image must not
/// be duplicated as the line grows.
#[test]
fn image_and_link_on_same_line() {
    let full = "prev\n\n![alt](https://img.com/i.png) [x][ref]\n\ntext\n";
    assert_converges(full, "image and link on same line");
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
    assert_converges(full, "reference before use");
}

/// A metadata block after content should not break the flow.
#[test]
fn metadata_after_content() {
    let full = "- a\n\n+++\nmeta: 1\n+++\n\n- b\n";
    assert_converges(full, "metadata after content");
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

    assert_converges(&full, "mixed document");
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

/// A list that spans the start of the re-parse window: while its
/// last line is being pushed, it is a lone paragraph that closes the
/// list; when it becomes a list item, the list must not be split in
/// two.
#[test]
fn list_spanning_the_reparse_window_is_not_split() {
    assert_converges(
        "2. two\n\n2. two\n",
        "loose list across the re-parse window",
    );
    assert_converges(
        "2. two\n2. two\n\n2. two\n",
        "list, loose list, across the re-parse window",
    );
}

/// A metadata block at the start of the stream is swallowed by the
/// parser once it is closed; the first line is a tentative rule
/// until then, so the whole block must be re-parsed as it grows.
#[test]
fn leading_metadata_block_is_swallowed() {
    assert_converges("---\nkey: value\n---\n\nmore\n", "a closed metadata block");
    assert_converges(
        "---\nkey: value\n...\n\nmore\n",
        "a metadata block closed by ellipsis",
    );
    assert_converges(
        "+++\nkey: value\n+++\n\nmore\n",
        "a plus-delimited metadata block",
    );
    // Not a metadata block: the second line is blank, so the first
    // line is a rule
    assert_converges("---\n\nmore\n", "a leading rule");
    // Not closed: the `---` is a rule and the block stays open
    assert_converges(
        "---\nkey: value\n--- x\n\nmore\n",
        "an unclosed metadata block",
    );
}

/// A metadata block in the middle of the stream is swallowed by the
/// parser once it is closed, too: its opening line is a tentative
/// rule until then, so the whole block must be re-parsed from where
/// it starts as it grows.
#[test]
fn mid_metadata_block_is_swallowed() {
    // After a blank line, and after a list item (no blank line)
    assert_converges(
        "para\n\n---\nkey: value\n---\n\nmore\n",
        "a metadata block after a blank line",
    );
    assert_converges(
        "- x\n---\nkey: value\n---\n\nmore\n",
        "a metadata block after a list item",
    );
    assert_converges(
        "- x\n---\nkey: value\n...\n\nmore\n",
        "a metadata block closed by ellipsis",
    );
    assert_converges(
        "para\n\n+++\nkey: value\n+++\n\nmore\n",
        "a plus-delimited metadata block",
    );
    // A list follows the block: the list must not be merged with the
    // tentative rule
    assert_converges(
        "- a\n- b\n\n---\nkey: value\n---\n\n- c\n",
        "a metadata block between two lists",
    );
}

#[test]
fn empty_input_has_no_sections() {
    let items: Vec<_> = parse("").collect();
    assert_eq!(sections(&items).count(), 0);
}

#[test]
fn input_without_headings_is_a_single_section() {
    let items: Vec<_> = parse("hello\n\nworld").collect();
    let [(heading, body)] = partition(&items);

    assert!(heading.is_none());
    assert_eq!(body.len(), items.len());
}

#[test]
fn prefix_before_first_heading() {
    let items: Vec<_> = parse("prefix\n# Heading\n\nbody\n\nand more").collect();
    let [(preamble_heading, preamble), (heading, body)] = partition(&items);

    assert!(preamble_heading.is_none());
    assert_eq!(preamble.len(), 1);
    assert!(std::ptr::eq(&preamble[0], &items[0]));

    assert!(std::ptr::eq(
        heading.expect("Expected a heading"),
        &items[1]
    ));
    assert_eq!(body.len(), 2);
    assert!(std::ptr::eq(&body[0], &items[2]));
    assert!(std::ptr::eq(&body[1], &items[3]));
}

#[test]
fn consecutive_headings_yield_empty_bodies() {
    let items: Vec<_> = parse("# A\n\n# B\n\n# C\n\nbody").collect();
    let [
        (heading_a, body_a),
        (heading_b, body_b),
        (heading_c, body_c),
    ] = partition(&items);

    assert!(heading_a.is_some());
    assert!(body_a.is_empty());
    assert!(heading_b.is_some());
    assert!(body_b.is_empty());
    assert!(heading_c.is_some());
    assert_eq!(body_c.len(), 1);
}

#[test]
fn trailing_heading_yields_an_empty_body() {
    let items: Vec<_> = parse("body\n# Heading").collect();
    let [(preamble_heading, preamble), (heading, body)] = partition(&items);

    assert!(preamble_heading.is_none());
    assert_eq!(preamble.len(), 1);
    assert!(heading.is_some());
    assert!(body.is_empty());
}

#[test]
fn every_item_is_yielded_exactly_once() {
    let items: Vec<_> =
        parse("intro\n# H1\n\np1\n- item\n> quote\n## H2\n\n```\ncode\n```\np2\n# H3").collect();
    let sections = sections(&items).collect::<Vec<_>>();

    // The headings are yielded as the first element of their sections,
    // in order
    assert_same_items(
        sections.iter().filter_map(|(heading, _)| *heading),
        items
            .iter()
            .filter(|item| matches!(item, Item::Heading(..))),
    );

    // The bodies partition the non-heading items, in order
    assert_same_items(
        sections.iter().flat_map(|(_, body)| body.iter()),
        items
            .iter()
            .filter(|item| !matches!(item, Item::Heading(..))),
    );
}

#[test]
fn content_sections() {
    let content = Content::parse("prefix\n# Heading\n\nbody");
    let [(preamble_heading, _), (heading, _)] = partition(content.items());

    assert!(preamble_heading.is_none());
    assert!(std::ptr::eq(
        heading.expect("Expected a heading"),
        &content.items()[1]
    ));
}

/// Fuzzes the convergence of the incremental parse over a corpus of
/// generated documents, with the three chunking schemes, checking the
/// one-shot equivalence after every push.
#[test]
fn fuzz() {
    /// The blocks from which the fuzzed documents are generated.
    const BLOCKS: &[&str] = &[
        "para\n\n",
        "text [x][ref] more\n\n",
        "- a\n",
        "- a\n- b\n",
        "- a [x][ref]\n",
        "- [x][ref]\n",
        "- b [y][ref]\n\n",
        "2. two\n",
        "2. two [y][ref]\n",
        "- [ ] task\n",
        "> - in quote\n",
        "> quote\n\n",
        "| a | b |\n| - | - |\n| 1 | 2 |\n\n",
        "| a | b |\n| - | - |\n| 1 | 2 | [x][ref] |\n\n",
        "[ref]: https://example.com\n\n",
        "[ref]: https://changed.com\n\n",
        "[other]: https://other.com\n\n",
        "[x]: https://x.com\n\n",
        "![alt](https://img.com/i.png)\n\n",
        "![alt with [x][ref]](https://img.com/i.png)\n\n",
        "before ![alt [x][ref]](https://img.com/i.png) trailing\n\n",
        "---\n\n",
        "---\nkey: value\n---\n\n",
        "+++\nkey: value\n+++\n\n",
        "---\nkey: value\n...\n\n",
        "---\nkey: value\n",
        "# heading\n\n",
        "## setext\n---\n\n",
        "```\ncode\n```\n\n",
        "\n",
    ];

    /// A deterministic pseudo-random generator.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> usize {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) as usize
        }
    }

    /// The documents to fuzz: the four convergence repros, and a
    /// deterministic corpus of documents generated from `BLOCKS`.
    fn corpus() -> impl Iterator<Item = String> {
        const REPROS: &[&str] = &[
            "[other]: https://other.com\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n\
         [ref]: https://example.com\n\n- a\n---\n\n",
            "2. two\n[ref]: https://changed.com\n\n\n- a\n| a | b |\n| - | - |\n| 1 | 2 |\n\n---\n\n",
            "[ref]: https://example.com\n\n> - in quote\n[ref]: https://changed.com\n\n\
         [ref]: https://changed.com\n\n> - in quote\n> - in quote\n- b [y][ref]\n\n",
            // A metadata block at the start of the document
            "---\nkey: value\n---\n\npara\n\n- a\n",
            // A metadata block in the middle of the document, after a
            // blank line and after a list item (no blank line), closed
            // and unclosed
            "para\n\n---\nkey: value\n---\n\nmore\n\n- a\n",
            "- x\n---\nkey\n---\npara\n",
            "- x\n---\nkey\n...\nmore\n",
        ];

        let mut rng = Rng(0x5EED);

        REPROS
            .iter()
            .copied()
            .map(str::to_owned)
            .chain((0..600).map(move |_| {
                let mut doc = String::new();

                for _ in 0..4 + rng.next() % 7 {
                    doc.push_str(BLOCKS[rng.next() % BLOCKS.len()]);
                }

                doc
            }))
    }

    for doc in corpus() {
        assert_converges(&doc, "fuzzed");
    }
}

/// Asserts that parsing `full` incrementally, with different chunking
/// schemes, converges to the one-shot parse.
fn assert_converges(full: &str, label: &str) {
    let word_by_word = full.split_inclusive(' ').map(str::to_owned).collect();
    let char_by_char = full.chars().map(|c| c.to_string()).collect();

    let three_char_chunks: Vec<_> = {
        let mut i = 0;

        std::iter::from_fn(|| {
            if i >= full.len() {
                return None;
            }

            let end = (i + 3).min(full.len());
            let chunk = full[i..end].to_owned();
            i = end;

            Some(chunk)
        })
        .collect()
    };

    for (chunks, name) in [
        (word_by_word, "word-by-word"),
        (char_by_char, "char-by-char"),
        (three_char_chunks, "3-char chunks"),
    ] {
        let mut stream = Content::new();

        for chunk in chunks {
            stream.push_str(&chunk);

            let one_shot = Content::parse(stream.raw());

            assert_eq!(
                format!("{:?}", stream.items()),
                format!("{:?}", one_shot.items()),
                "{label}: incremental ({name}) should converge to one-shot\n\n{raw}",
                raw = stream.raw(),
            );

            assert_eq!(
                stream.images(),
                one_shot.images(),
                "{label}: images should converge ({name})\n\n{raw}",
                raw = stream.raw(),
            );
        }
    }
}

/// Asserts that two iterators yield the exact same [`Item`]
/// references, in the same order.
fn assert_same_items<'a>(
    left: impl IntoIterator<Item = &'a Item>,
    right: impl IntoIterator<Item = &'a Item>,
) {
    let (mut left, mut right) = (left.into_iter(), right.into_iter());

    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) => assert!(std::ptr::eq(left, right)),
            (None, None) => break,
            (left, right) => panic!("Length mismatch: {left:?} vs {right:?}"),
        }
    }
}

/// The sections yielded by [`sections`], expecting exactly `N` of them.
fn partition<const N: usize>(items: &[Item]) -> [(Option<&Item>, &[Item]); N] {
    sections(items)
        .collect::<Vec<_>>()
        .try_into()
        .expect("Unexpected number of sections")
}
