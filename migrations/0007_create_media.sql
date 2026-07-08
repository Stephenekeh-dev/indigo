CREATE TYPE post_status AS ENUM ('draft', 'published', 'archived');
CREATE TYPE post_category AS ENUM (
    'rust_basics', 'systems_programming', 'blockchain',
    'career', 'project_showcase', 'news', 'tutorial'
);

CREATE TABLE posts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    author_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           VARCHAR(300) NOT NULL,
    slug            VARCHAR(320) NOT NULL UNIQUE,
    excerpt         VARCHAR(500),
    content         TEXT NOT NULL,
    status          post_status NOT NULL DEFAULT 'draft',
    category        post_category NOT NULL DEFAULT 'rust_basics',
    cover_image_url TEXT,
    tags            TEXT[] DEFAULT '{}',
    read_time_mins  INT,
    view_count      INT NOT NULL DEFAULT 0,
    likes_count     INT NOT NULL DEFAULT 0,
    seo_title       VARCHAR(70),
    seo_description VARCHAR(160),
    published_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE post_likes (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(post_id, user_id)
);

CREATE TABLE post_comments (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id   UUID REFERENCES post_comments(id) ON DELETE CASCADE,
    content     TEXT NOT NULL,
    is_approved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE newsletter_subscribers (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           VARCHAR(255) NOT NULL UNIQUE,
    full_name       VARCHAR(150),
    is_confirmed    BOOLEAN NOT NULL DEFAULT FALSE,
    confirm_token   TEXT UNIQUE,
    unsubscribed_at TIMESTAMPTZ,
    source          VARCHAR(100),
    subscribed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE newsletter_campaigns (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subject         VARCHAR(200) NOT NULL,
    preview_text    VARCHAR(200),
    content         TEXT NOT NULL,
    sent_count      INT NOT NULL DEFAULT 0,
    open_count      INT NOT NULL DEFAULT 0,
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_status       ON posts(status);
CREATE INDEX idx_posts_category     ON posts(category);
CREATE INDEX idx_posts_published_at ON posts(published_at DESC);
CREATE INDEX idx_posts_author       ON posts(author_id);
CREATE INDEX idx_comments_post      ON post_comments(post_id);
CREATE INDEX idx_subscribers_email  ON newsletter_subscribers(email);

CREATE TRIGGER posts_updated_at
    BEFORE UPDATE ON posts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER post_comments_updated_at
    BEFORE UPDATE ON post_comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();