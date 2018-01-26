
use super::*;


/// Used for grouping entries in the output
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum WordClassesGroup {
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DictQueryResultGrouped {
    word_count_groups: Vec<DictEntryWordCountGroup>
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

        let mut format = FORMAT_CLEAN.clone();

        format.separator(Intern, LineSeparator::new(' ', ' ', ' ', ' '));
        format.padding(0, 0);

        table.set_format(format);

        f.write_str(&table.to_string())
    }
}

impl From<DictQueryResult> for DictQueryResultGrouped {
    fn from(query_result: DictQueryResult) -> Self {
        use itertools::Itertools;

        let mut entries = query_result.entries;

        // TODO: left/right
        entries.sort_unstable_by_key(DictEntry::get_max_word_count);

        let word_count_group_by = entries.into_iter().group_by(DictEntry::get_max_word_count);

        let grouped_entries: Vec<_> = word_count_group_by.into_iter().map(|(word_count, same_word_count_group)| {
            let mut same_word_count_pairs: Vec<(WordClassesGroup, DictEntry)> = same_word_count_group
                .map(|entry| {
                    let word_classes_group: WordClassesGroup = entry.word_classes.as_slice().into();

                    (word_classes_group, entry)
                })
                .collect();

            same_word_count_pairs.sort_unstable_by_key(|&(word_class_group, _)| word_class_group);

            let word_class_group_by =
                same_word_count_pairs.into_iter().group_by(|&(word_class_group, _)| word_class_group);

            let vec_word_class_group: Vec<DictEntryWordClassGroup> =
                word_class_group_by.into_iter().map(|(word_class_group, entries_group)| {
                    let mut entries: Vec<DictEntry> = entries_group.map(|(_, entry)| entry).collect();

                    // TODO: left/right => left:left;right:right;bi:left
                    entries.sort_by(|left_entry, right_entry| {
                        let left = &left_entry.source.indexed_word;
                        let right = &right_entry.source.indexed_word;

                        left.cmp(&right)
                    });

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

#[derive(Clone, Eq, PartialEq, Debug)]
struct DictEntryWordCountGroup {
    word_count: u8,
    word_class_groups: Vec<DictEntryWordClassGroup>,
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

        let mut format = FORMAT_CLEAN.clone();

        format.separator(Intern, LineSeparator::new(' ', ' ', ' ', ' '));
        format.padding(0, 0);

        table.set_format(format);

        f.write_str(&table.to_string())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct DictEntryWordClassGroup {
    word_count: u8,
    word_class_group: WordClassesGroup,
    entries: Vec<DictEntry>,
}

impl Display for DictEntryWordClassGroup {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use prettytable;
        use prettytable::Table;

        // TODO:
        // Complete rendering of word
        // Colored
        let entry_rows: Vec<_> = self.entries.iter().map(|entry| {
            let left = &entry.source;
            let right = &entry.translation;

            row![left, right]
        }).collect();

        let mut entry_table = Table::init(entry_rows);

        entry_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        let entry_table_string = entry_table.to_string();

        let header_string = match self.word_count {
            0 | 1 => format!("{:?}", self.word_class_group),
            higher_word_count => format!("{} Words: {:?}", higher_word_count, self.word_class_group),
        };

        let mut complete_table = Table::init(vec![row![header_string], row![entry_table_string]]);


        let mut format = prettytable::format::consts::FORMAT_NO_BORDER.clone();

        format.padding(0, 0);

        complete_table.set_format(format);

        f.write_str(&complete_table.to_string())
    }
}