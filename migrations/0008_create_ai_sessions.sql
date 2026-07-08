CREATE TYPE ai_message_role AS ENUM ('user', 'assistant', 'system');
CREATE TYPE ai_session_context AS ENUM (
    'general', 'rust_help', 'course_support',
    'booking_inquiry', 'sales', 'enterprise_inquiry'
);

CREATE TABLE ai_sessions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    session_token   TEXT NOT NULL UNIQUE,
    context         ai_session_context NOT NULL DEFAULT 'general',
    title           VARCHAR(200),
    message_count   INT NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    ip_address      INET,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_messages (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id  UUID NOT NULL REFERENCES ai_sessions(id) ON DELETE CASCADE,
    role        ai_message_role NOT NULL,
    content     TEXT NOT NULL,
    tokens_used INT,
    model       VARCHAR(60),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_usage_logs (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id          UUID REFERENCES ai_sessions(id) ON DELETE SET NULL,
    user_id             UUID REFERENCES users(id) ON DELETE SET NULL,
    prompt_tokens       INT NOT NULL DEFAULT 0,
    completion_tokens   INT NOT NULL DEFAULT 0,
    total_tokens        INT NOT NULL DEFAULT 0,
    model               VARCHAR(60),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_sessions_user    ON ai_sessions(user_id);
CREATE INDEX idx_ai_sessions_token   ON ai_sessions(session_token);
CREATE INDEX idx_ai_messages_session ON ai_messages(session_id);
CREATE INDEX idx_ai_usage_user       ON ai_usage_logs(user_id);

CREATE TRIGGER ai_sessions_updated_at
    BEFORE UPDATE ON ai_sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();