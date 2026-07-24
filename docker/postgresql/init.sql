-- Add migration script here

-- =========================
-- users 表
-- =========================
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    ws_id bigint NOT NULL ,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- hashed argon2 password, length 97
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- workspace for users
CREATE TABLE IF NOT EXISTS workspaces(
  id bigserial PRIMARY KEY,
  name varchar(32) NOT NULL UNIQUE,
  owner_id bigint NOT NULL REFERENCES users(id),
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

BEGIN;
INSERT INTO users(id, ws_id, fullname, email, password_hash)
  VALUES (0, 0, 'super user', 'super@none.org', '');
INSERT INTO workspaces(id, name, owner_id)
  VALUES (0, 'none', 0);
COMMIT;
-- add foreign key constraint for ws_id for users
ALTER TABLE users
  ADD CONSTRAINT users_ws_id_fk FOREIGN KEY (ws_id) REFERENCES workspaces(id);

-- unique index on email
CREATE UNIQUE INDEX IF NOT EXISTS email_index
ON users(email);


-- =========================
-- chat_type 枚举类型
-- =========================
CREATE TYPE chat_type AS ENUM (
    'single',
    'group',
    'private_channel',
    'public_channel'
);


-- =========================
-- chats 表
-- =========================
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(64),
    type chat_type NOT NULL,
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


-- =========================
-- messages 表
-- =========================
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    files TEXT[] DEFAULT '{}', -- 存储文件的 URL 列表
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 索引：按 chat_id + 时间倒序
CREATE INDEX IF NOT EXISTS chat_id_created_at_index
ON messages(chat_id, created_at DESC);

-- 索引：按 sender_id + 时间倒序
CREATE INDEX IF NOT EXISTS sender_id_index
ON messages(sender_id, created_at DESC);
