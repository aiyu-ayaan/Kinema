-- Library counts (/home, /libraries) read only this index instead of every item row.
CREATE INDEX items_lib_parent_kind ON items(library_id, parent_id, kind);
-- "Recently added" stops after its LIMIT: the index already has the (added_at, id) order, so no sort of every row
-- added in the same second (a scan adds a whole folder at once).
CREATE INDEX items_lib_added_id ON items(library_id, added_at DESC, id DESC);
