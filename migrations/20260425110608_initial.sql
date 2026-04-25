-- Add migration script here

-- =========================
-- users 表
-- =========================
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- hashed argon2 password, length 97
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 唯一索引：email
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
    name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
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
    images TEXT[],
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 索引：按 chat_id + 时间倒序
CREATE INDEX IF NOT EXISTS chat_id_created_at_index
ON messages(chat_id, created_at DESC);

-- 索引：按 sender_id + 时间倒序
CREATE INDEX IF NOT EXISTS sender_id_index
ON messages(sender_id, created_at DESC);
