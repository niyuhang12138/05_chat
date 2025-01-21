-- install workspace
INSERT INTO workspace(name, owner_id) VALUES('acme', 0), ('foo', 0), ('bar', 0);

-- insert users
INSERT INTO users(ws_id, fullname, email, password_hash) VALUES
(1, 'nyh', 'nyh@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$cIBnY9un3yp9u01Qt3zFlQ$bVUpm1w0q+clSGgIlITlmYk6uun3PHxMa6xO1E7RW1M'),
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
