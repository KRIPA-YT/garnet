CREATE POLICY "Owners and editors can update items"
ON items
FOR UPDATE
USING (
    EXISTS (
        SELECT 1
        FROM members m
        WHERE m.list_id = items.list_id
          AND m.user_id = current_setting('app.user_id')::uuid
          AND m.role >= 'editor'
    )
)
WITH CHECK (
    EXISTS (
        SELECT 1
        FROM members m
        WHERE m.list_id = items.list_id
          AND m.user_id = current_setting('app.user_id')::uuid
          AND m.role >= 'editor'
    )
);
