SELECT
  *,
  highlight(:dict_id, 0, '{', '}') AS highlight_left_indexed_word,
  highlight(:dict_id, 1, '{', '}') AS highlight_right_indexed_word
FROM :dict_id
WHERE :dict_id MATCH ?
ORDER BY bm25(:dict_id);
