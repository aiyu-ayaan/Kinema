-- A link with `max_uses` set admits that many distinct viewers (by client address, hashed with the link id);
-- a viewer already admitted can come back, so a page reload never costs a use.
CREATE TABLE share_viewers (
  share_id INTEGER NOT NULL REFERENCES share_links(id) ON DELETE CASCADE,
  viewer TEXT NOT NULL,
  first_seen INTEGER NOT NULL,
  PRIMARY KEY (share_id, viewer)
);
