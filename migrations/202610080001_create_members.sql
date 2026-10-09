CREATE TYPE role AS ENUM ('owner', 'editor', 'viewer');

CREATE TABLE members (
    list_id UUID NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role ROLE NOT NULL DEFAULT 'viewer',
    PRIMARY KEY (list_id, user_id)
);

CREATE UNIQUE INDEX one_owner_per_list ON members(list_id) WHERE role = 'owner';
