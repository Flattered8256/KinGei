ALTER TABLE friend DROP CONSTRAINT friend_user_id_fkey;
ALTER TABLE friend DROP CONSTRAINT friend_friend_id_fkey;
ALTER TABLE friend
    ADD CONSTRAINT friend_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE;
ALTER TABLE friend
    ADD CONSTRAINT friend_friend_id_fkey
    FOREIGN KEY (friend_id) REFERENCES "user"(id) ON DELETE CASCADE;

ALTER TABLE friend_request DROP CONSTRAINT friend_request_sender_id_fkey;
ALTER TABLE friend_request DROP CONSTRAINT friend_request_receiver_id_fkey;
ALTER TABLE friend_request
    ADD CONSTRAINT friend_request_sender_id_fkey
    FOREIGN KEY (sender_id) REFERENCES "user"(id) ON DELETE CASCADE;
ALTER TABLE friend_request
    ADD CONSTRAINT friend_request_receiver_id_fkey
    FOREIGN KEY (receiver_id) REFERENCES "user"(id) ON DELETE CASCADE;

ALTER TABLE chat DROP CONSTRAINT chat_owner_id_fkey;
ALTER TABLE chat
    ADD CONSTRAINT chat_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES "user"(id) ON DELETE SET NULL;

ALTER TABLE chat_member DROP CONSTRAINT chat_member_user_id_fkey;
ALTER TABLE chat_member DROP CONSTRAINT chat_member_chat_id_fkey;
ALTER TABLE chat_member
    ADD CONSTRAINT chat_member_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES "user"(id) ON DELETE CASCADE;
ALTER TABLE chat_member
    ADD CONSTRAINT chat_member_chat_id_fkey
    FOREIGN KEY (chat_id) REFERENCES chat(id) ON DELETE CASCADE;

ALTER TABLE file DROP CONSTRAINT file_owner_id_fkey;
ALTER TABLE file
    ADD CONSTRAINT file_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES "user"(id) ON DELETE CASCADE;

ALTER TABLE message DROP CONSTRAINT message_sender_id_fkey;
ALTER TABLE message DROP CONSTRAINT message_chat_id_fkey;
ALTER TABLE message
    ADD CONSTRAINT message_sender_id_fkey
    FOREIGN KEY (sender_id) REFERENCES "user"(id) ON DELETE CASCADE;
ALTER TABLE message
    ADD CONSTRAINT message_chat_id_fkey
    FOREIGN KEY (chat_id) REFERENCES chat(id) ON DELETE CASCADE;
