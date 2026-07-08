CREATE TYPE product_type AS ENUM (
    'ebook', 'template', 'tool', 'merch', 'course_bundle', 'other'
);
CREATE TYPE product_status AS ENUM ('active', 'draft', 'archived');
CREATE TYPE order_status AS ENUM (
    'pending', 'paid', 'fulfilled', 'refunded', 'cancelled'
);

CREATE TABLE products (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title           VARCHAR(200) NOT NULL,
    slug            VARCHAR(220) NOT NULL UNIQUE,
    description     TEXT NOT NULL,
    short_desc      VARCHAR(400),
    product_type    product_type NOT NULL,
    status          product_status NOT NULL DEFAULT 'draft',
    price_usd       NUMERIC(10,2) NOT NULL,
    compare_price   NUMERIC(10,2),
    is_digital      BOOLEAN NOT NULL DEFAULT TRUE,
    download_url    TEXT,
    thumbnail_url   TEXT,
    images          TEXT[] DEFAULT '{}',
    tags            TEXT[] DEFAULT '{}',
    stock_count     INT,
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status              order_status NOT NULL DEFAULT 'pending',
    total_usd           NUMERIC(10,2) NOT NULL,
    stripe_payment_id   TEXT,
    stripe_session_id   TEXT,
    currency            VARCHAR(10) NOT NULL DEFAULT 'usd',
    billing_email       VARCHAR(255),
    notes               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE order_items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id    UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id  UUID REFERENCES products(id) ON DELETE SET NULL,
    title       VARCHAR(200) NOT NULL,
    price_usd   NUMERIC(10,2) NOT NULL,
    quantity    INT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE cart_items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id  UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity    INT NOT NULL DEFAULT 1,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, product_id)
);

CREATE TABLE discount_codes (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code                VARCHAR(50) NOT NULL UNIQUE,
    description         TEXT,
    discount_pct        NUMERIC(5,2),
    discount_flat_usd   NUMERIC(10,2),
    max_uses            INT,
    used_count          INT NOT NULL DEFAULT 0,
    expires_at          TIMESTAMPTZ,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_status   ON products(status);
CREATE INDEX idx_products_type     ON products(product_type);
CREATE INDEX idx_orders_user       ON orders(user_id);
CREATE INDEX idx_orders_status     ON orders(status);
CREATE INDEX idx_order_items_order ON order_items(order_id);
CREATE INDEX idx_cart_user         ON cart_items(user_id);

CREATE TRIGGER products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();