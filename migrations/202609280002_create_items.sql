CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id UUID NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    title varchar(255) NOT NULL,
    checked bool NOT NULL DEFAULT false,
    pinned bool NOT NULL DEFAULT false
)
