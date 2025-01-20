-- Add migration script here
-- if chat changed, notify with chat data
CREATE OR REPLACE FUNCTION add_to_chat()
  RETURNS TRIGGER
  AS $$
BEGIN
  RAISE NOTICE 'add_to_chat: %', NEW;
  PERFORM
    pg_notify('chat_updated', json_build_object('op', TG_OP, 'old', OLD, 'new', NEW)::text);
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER add_to_chat_trigger
  AFTER INSERT OR UPDATE OR DELETE ON chats
  FOR EACH ROW
  EXECUTE FUNCTION add_to_chat();

-- if new message added, notify with message data
CREATE OR REPLACE FUNCTION add_to_message()
  RETURNS TRIGGER
  AS $$
DECLARE
  USERS bigint[];
BEGIN
  IF TG_OP = 'INSERT' THEN
    RAISE NOTICE 'add_to_message: %', NEW;
    -- select chat with chat_id in NEW
    SELECT
      members INTO USERS
    FROM
      chats
    WHERE
      id = NEW.chat_id;
    PERFORM
      pg_notify('chat_message_created', json_build_object('message', NEW, 'members', USERS)::text);
  END IF;
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER add_to_message_trigger
  AFTER INSERT ON messages
  FOR EACH ROW
  EXECUTE FUNCTION add_to_message();


-- install workspace
INSERT INTO workspace(name, owner_id) VALUES('acme', 0), ('foo', 0), ('bar', 0);

-- insert users
INSERT INTO users(ws_id, fullname, email, password_hash) VALUES
(1, 'alice', 'alice123', '$argon2id$v=19$m=19456,t=2,p=1$okFsCVybSHhdYawpO7YJ8Q$0vIxwL7JtjBggJ2+WhSvGqvyDgVit2Hc8mGoiCNH2uQ'),
(1, 'bob', 'bob123', '$argon2id$v=19$m=19456,t=2,p=1$okFsCVybSHhdYawpO7YJ8Q$0vIxwL7JtjBggJ2+WhSvGqvyDgVit2Hc8mGoiCNH2uQ'),
(1, 'join', 'join123', '$argon2id$v=19$m=19456,t=2,p=1$okFsCVybSHhdYawpO7YJ8Q$0vIxwL7JtjBggJ2+WhSvGqvyDgVit2Hc8mGoiCNH2uQ'),
(1, 'black', 'black123', '$argon2id$v=19$m=19456,t=2,p=1$okFsCVybSHhdYawpO7YJ8Q$0vIxwL7JtjBggJ2+WhSvGqvyDgVit2Hc8mGoiCNH2uQ'),
(1, 'charlie', 'charlie123', '$argon2id$v=19$m=19456,t=2,p=1$okFsCVybSHhdYawpO7YJ8Q$0vIxwL7JtjBggJ2+WhSvGqvyDgVit2Hc8mGoiCNH2uQ');

-- insert
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members) VALUES
  (1, 'general', 'public_channel', '{1,2,3,4,5}'),
  (1, 'private', 'private_channel', '{2,3,4}');

-- insert unnamed chat
INSERT INTO chats (ws_id, type, members) VALUES
  (1, 'single', '{2,3}'),
  (1, 'group', '{3,4,6}');


-- insert messages
INSERT INTO messages(chat_id, sender_id, content)
  VALUES (1, 1, 'Hello, world!'),
(1, 2, 'Hi, there!'),
(1, 3, 'How are you?'),
(1, 4, 'I am fine, thank you!'),
(1, 5, 'Good to hear that!'),
(1, 1, 'Hello, world!'),
(1, 2, 'Hi, there!'),
(1, 3, 'How are you?'),
(1, 1, 'Hello, world!'),
(1, 1, 'Hello, world!');
