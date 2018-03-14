DROP TABLE IF EXISTS dictionaries;

CREATE TABLE dictionaries(
  dict_id TEXT UNIQUE NOT NULL,
  left_language TEXT NOT NULL,
  right_language TEXT NOT NULL,
  path TEXT NOT NULL
  --   date TEXT NOT NULL
);