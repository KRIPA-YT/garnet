CREATE POLICY "Members can view items"
ON items
FOR SELECT
USING (
    EXISTS (
        SELECT 1
        FROM members m
        WHERE m.list_id = items.list_id
          AND m.user_id = current_setting('app.user_id')::uuid
    )
);
