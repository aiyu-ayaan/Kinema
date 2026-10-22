-- Franchise groups. `tmdb_id` is set for groups TMDB reported (belongs_to_collection), NULL for manual ones.
CREATE TABLE collections (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  tmdb_id INTEGER UNIQUE,
  created_at INTEGER NOT NULL
);
CREATE TABLE collection_items (
  collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  PRIMARY KEY (collection_id, item_id)
);
CREATE INDEX collection_items_item ON collection_items(item_id);

-- Home shelves: a saved genre query with a title. Admin defines them, each profile may hide any.
CREATE TABLE shelves (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  genre TEXT NOT NULL,
  position INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
CREATE TABLE shelf_hidden (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  shelf_id INTEGER NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, shelf_id)
);
INSERT INTO shelves (title, genre, position, created_at) VALUES ('Oh, The Horror!', 'Horror', 0, 0);
