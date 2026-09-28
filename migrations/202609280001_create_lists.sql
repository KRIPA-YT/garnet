-- migrations/202609280001_create_lists.sql

CREATE TABLE lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title varchar(255) NOT NULL,
    pinned BOOLEAN NOT NULL DEFAULT false
);
