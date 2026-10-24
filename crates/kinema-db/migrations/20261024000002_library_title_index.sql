-- Library pages sorted by title read this index in order and stop after their LIMIT instead of sorting the library.
CREATE INDEX items_lib_top_title ON items(library_id, parent_id, sort_title, id);
