-- this file is used for postgresql database initialization
-- crate user table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- password aron2 password hash
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- crate index for user for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- crate chat type signal, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('signal', 'group', 'private_channel', 'public_channel');

-- crate chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- crate message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (sender_id) REFERENCES users(id)
);

-- crate index for message for chat_id and crated_at order by created_at desc
CREATE INDEX IF NOT EXISTS chat_id_created_at_idx ON messages(chat_id, created_at DESC);

-- create index IF NOT EXISTS sender_id_idx
CREATE INDEX IF NOT EXISTS sender_id_idx ON messages(sender_id);
