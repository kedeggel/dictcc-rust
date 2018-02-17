SELECT
  *,
  highlight(entries, 0, '{', '}') AS highlight_left_indexed_word,
  highlight(entries, 1, '{', '}') AS highlight_right_indexed_word
FROM entries e
WHERE entries MATCH ?
ORDER BY bm25(entries);
