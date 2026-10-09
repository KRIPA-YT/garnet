CREATE POLICY "Members can view lists"
ON lists
FOR SELECT
USING (
    EXISTS (
        SELECT 1
        FROM members m
        WHERE m.list_id = lists.id
          AND m.user_id = current_setting('app.user_id')::uuid
    )
);
