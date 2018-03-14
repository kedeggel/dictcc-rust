DROP TABLE IF EXISTS :dict_id;

CREATE VIRTUAL TABLE :dict_id
  USING fts5(
      left_indexed_word,
      right_indexed_word,
      left_word UNINDEXED,
      right_word UNINDEXED,
      word_classes UNINDEXED
  );
