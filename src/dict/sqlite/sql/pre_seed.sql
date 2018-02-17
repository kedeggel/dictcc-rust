DROP TABLE entries;

CREATE VIRTUAL TABLE entries
  USING fts5(
      left_indexed_word,
      right_indexed_word,
      left_word UNINDEXED,
      right_word UNINDEXED,
      word_classes UNINDEXED
  );