//! Grouped representation and pretty printing of `DictQueryResult` in tabular form.
//!
//! The grouping is inspired by the [result page of dict.cc](https://www.dict.cc/?s=house).
//!
//! The grouping has two layers:
//! 1. By word count
//! 2. By word class group.
//!
//! # Example Output
//!
//! ```ignore
//! Verbs
//! --------------------
//!  Verb | verb | Verb
//!
//! Nouns
//! --------------------------
//!  DE         | EN   | Noun
//!  foo        | foo  | Noun
//!  Substantiv | noun | Noun
//!
//! Others
//! -------------------
//!  a | c | Adjective
//!  B | B | Adjective
//!  c | a | Adjective
//!
//! 2 Words: Verbs
//! ----------------------------
//!  foo Verb | foo verb | Verb
//! ```

use dict::query::DictQueryResult;
use dict::query::QueryDirection;
use itertools::GroupBy;
use itertools::Itertools;
use std::vec::IntoIter;
use super::*;

/// Coarse grouping of `WordClass`.
#[allow(missing_docs)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum WordClassesGroup {
    Verbs,
    Nouns,
    Others,
}

impl<'a> From<&'a [WordClass]> for WordClassesGroup {
    fn from(word_classes: &[WordClass]) -> Self {
        use self::WordClassesGroup::*;

        if word_classes.contains(&WordClass::Verb) {
            Verbs
        } else if word_classes.contains(&WordClass::Noun) {
            Nouns
        } else {
            Others
        }
    }
}

/// Grouped representation of `DictQueryResult`.
///
/// Implements Display using a formatted and aligned table.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResultGrouped {
    word_count_groups: Vec<DictEntryWordCountGroup>
}

impl DictQueryResultGrouped {
    /// Returns a slice of `DictEntryWordCountGroup`.
    pub fn word_count_groups(&self) -> &[DictEntryWordCountGroup] {
        &self.word_count_groups
    }
}

impl Display for DictQueryResultGrouped {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use prettytable::format::LinePosition::*;
        use prettytable::format::LineSeparator;
        use prettytable::format::consts::FORMAT_CLEAN;
        use prettytable::Table;

        let mut table = Table::init(
            self.word_count_groups.iter()
                .map(|word_count_group| row!(word_count_group.to_string()))
                .collect()
        );

        let mut format = *FORMAT_CLEAN;

        format.separator(Intern, LineSeparator::new(' ', ' ', ' ', ' '));
        format.padding(0, 0);

        table.set_format(format);

        f.write_str(&table.to_string())
    }
}


impl From<DictQueryResult> for DictQueryResultGrouped {
    fn from(query_result: DictQueryResult) -> Self {
        fn group_entries<G, I>(mut entries: Vec<I>, by: fn(&I) -> G)
                               -> GroupBy<G, IntoIter<I>, fn(&I) -> G>
            where G: Ord {
            entries.sort_unstable_by_key(by);

            entries.into_iter().group_by(by)
        }

        let query_direction = query_result.query_direction;
        let entries = query_result.entries;

        let get_word_count: fn(&DictEntry) -> u8 = match query_direction {
            QueryDirection::ToRight => |entry: &DictEntry| {
                entry.left_word.word_count
            },
            QueryDirection::Bidirectional => DictEntry::get_max_word_count,
            QueryDirection::ToLeft => |entry: &DictEntry| {
                entry.right_word.word_count
            },
        };

        let word_count_group_by =
            group_entries(entries, get_word_count);

        let grouped_entries: Vec<_> = word_count_group_by.into_iter().map(|(word_count, same_word_count_group)| {
            let same_word_count_pairs: Vec<(WordClassesGroup, DictEntry)> = same_word_count_group
                .map(|entry| {
                    let word_classes_group: WordClassesGroup = entry.word_classes.as_slice().into();

                    (word_classes_group, entry)
                })
                .collect();

            let word_class_group_by =
                group_entries(same_word_count_pairs, |&(word_class_group, _)| word_class_group);

            let vec_word_class_group: Vec<DictEntryWordClassGroup> =
                word_class_group_by.into_iter().map(|(word_class_group, entries_group)| {
                    let mut entries: Vec<DictEntry> = entries_group.map(|(_, entry)| entry).collect();

                    let cmp_left = |left_entry: &DictEntry, right_entry: &DictEntry| {
                        let left = &left_entry.left_word.indexed_word;
                        let right = &right_entry.left_word.indexed_word;

                        left.cmp(right)
                    };

                    let cmp_right = |left_entry: &DictEntry, right_entry: &DictEntry| {
                        let left = &left_entry.right_word.indexed_word;
                        let right = &right_entry.right_word.indexed_word;

                        left.cmp(right)
                    };

                    match query_direction {
                        QueryDirection::ToRight |
                        QueryDirection::Bidirectional => entries.sort_by(cmp_left),
                        QueryDirection::ToLeft => entries.sort_by(cmp_right),
                    };

                    DictEntryWordClassGroup {
                        word_count,
                        word_class_group,
                        entries,
                    }
                }).collect();

            DictEntryWordCountGroup {
                word_count,
                word_class_groups: vec_word_class_group,
            }
        }).collect();

        DictQueryResultGrouped {
            word_count_groups: grouped_entries,
        }
    }
}

/// A group of entries, which have the same word count and are coarsely grouped by word class.
///
/// Implements Display using a formatted and aligned table.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictEntryWordCountGroup {
    word_count: u8,
    word_class_groups: Vec<DictEntryWordClassGroup>,
}

impl DictEntryWordCountGroup {
    /// Returns a slice of `DictEntryWordClassGroup`.
    pub fn word_class_groups(&self) -> &[DictEntryWordClassGroup] {
        &self.word_class_groups
    }

    /// The word count of this group.
    pub fn word_count(&self) -> u8 {
        self.word_count
    }
}

impl Display for DictEntryWordCountGroup {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use prettytable::format::LinePosition::*;
        use prettytable::format::LineSeparator;
        use prettytable::format::consts::FORMAT_CLEAN;
        use prettytable::Table;

        let mut table = Table::init(
            self.word_class_groups.iter()
                .map(|word_class_group| row!(word_class_group.to_string()))
                .collect()
        );

        let mut format = *FORMAT_CLEAN;

        format.separator(Intern, LineSeparator::new(' ', ' ', ' ', ' '));
        format.padding(0, 0);

        table.set_format(format);

        f.write_str(&table.to_string())
    }
}

/// A group of entries, which have the same word count and word class group.
///
/// Implements Display using a formatted and aligned table.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictEntryWordClassGroup {
    word_count: u8,
    word_class_group: WordClassesGroup,
    entries: Vec<DictEntry>,
}

impl DictEntryWordClassGroup {
    /// Returns a slice of entries in this group.
    pub fn entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// The word count of this group.
    pub fn word_count(&self) -> u8 {
        self.word_count
    }

    /// The word class group of this entry group.
    pub fn word_class_group(&self) -> WordClassesGroup {
        self.word_class_group
    }
}


impl Display for DictEntryWordClassGroup {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use prettytable;
        use prettytable::Table;

        // TODO: word classes filter (redundant classes)
        let entry_rows: Vec<_> = self.entries.iter().map(|entry| {
            let left = &entry.left_word.to_colored_string();
            let right = &entry.right_word.to_colored_string();

            let word_classes = &entry.word_classes.iter().map(|word_class| format!("{:?}", word_class)).collect::<Vec<_>>().join(", ");

            row![left, right, word_classes]
        }).collect();

        let mut entry_table = Table::init(entry_rows);

        entry_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        let entry_table_string = entry_table.to_string();

        let header_string = match self.word_count {
            0 | 1 => format!("{:?}", self.word_class_group),
            higher_word_count => format!("{} Words: {:?}", higher_word_count, self.word_class_group),
        };

        let mut complete_table = Table::init(vec![row![header_string], row![entry_table_string]]);


        let mut format = *prettytable::format::consts::FORMAT_NO_BORDER;

        format.padding(0, 0);

        complete_table.set_format(format);

        f.write_str(&complete_table.to_string())
    }
}
