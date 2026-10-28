-- The site owner: the first administrator, who can never lose the role or be deleted
-- (users-and-sharing.md). Existing profiles keep their is_admin as it is; only the
-- earliest administrator is marked owner.
ALTER TABLE users ADD COLUMN is_owner INTEGER NOT NULL DEFAULT 0;
UPDATE users SET is_owner = 1 WHERE id = (SELECT MIN(id) FROM users WHERE is_admin = 1);

-- The API refuses these with a clear message; the triggers keep any other path honest.
CREATE TRIGGER users_owner_stays_admin BEFORE UPDATE OF is_admin ON users
WHEN OLD.is_owner = 1 AND NEW.is_admin = 0
BEGIN
  SELECT RAISE(ABORT, 'the site owner stays an administrator');
END;

CREATE TRIGGER users_owner_not_deleted BEFORE DELETE ON users
WHEN OLD.is_owner = 1
BEGIN
  SELECT RAISE(ABORT, 'the site owner cannot be deleted');
END;
