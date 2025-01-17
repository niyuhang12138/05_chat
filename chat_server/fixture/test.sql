-- install workspace
INSERT INTO workspace(name, owner_id) VALUES('acme', 0), ('foo', 0), ('bar', 0);

-- insert users
INSERT INTO users(ws_id, fullname, email, password_hash) VALUES
(1, 'alice', 'alice123', '$argon2id$v=19$m=19456,t=2,p=1$MX7s8fQa+/6F0IzHcvsJpQ$4zbSkfughRqcE0iRO+cbb6fkiBIqfup0ZrxZsIBAg8I'),
(1, 'bob', 'bob123', '$argon2id$v=19$m=19456,t=2,p=1$MX7s8fQa+/6F0IzHcvsJpQ$4zbSkfughRqcE0iRO+cbb6fkiBIqfup0ZrxZsIBAg8I'),
(1, 'join', 'join123', '$argon2id$v=19$m=19456,t=2,p=1$MX7s8fQa+/6F0IzHcvsJpQ$4zbSkfughRqcE0iRO+cbb6fkiBIqfup0ZrxZsIBAg8I'),
(1, 'black', 'black123', '$argon2id$v=19$m=19456,t=2,p=1$MX7s8fQa+/6F0IzHcvsJpQ$4zbSkfughRqcE0iRO+cbb6fkiBIqfup0ZrxZsIBAg8I'),
(1, 'charlie', 'charlie123', '$argon2id$v=19$m=19456,t=2,p=1$MX7s8fQa+/6F0IzHcvsJpQ$4zbSkfughRqcE0iRO+cbb6fkiBIqfup0ZrxZsIBAg8I');

-- insert
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members) VALUES
  (1, 'general', 'public_channel', '{1,2,3,4,5}'),
  (1, 'private', 'private_channel', '{1,2,3}');

-- insert unnamed chat
INSERT INTO chats (ws_id, type, members) VALUES
  (1, 'signal', '{1,2}'),
  (1, 'group', '{1,3,4}');
