CREATE TYPE service_type AS ENUM (
    'migration', 'custom_app', 'code_review',
    'general_consulting', 'retainer', 'blockchain_consulting'
);
CREATE TYPE booking_status AS ENUM (
    'pending', 'confirmed', 'cancelled', 'completed', 'no_show'
);
CREATE TYPE project_status AS ENUM (
    'inquiry', 'proposal_sent', 'in_progress', 'completed', 'cancelled'
);

CREATE TABLE service_listings (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title           VARCHAR(200) NOT NULL,
    slug            VARCHAR(220) NOT NULL UNIQUE,
    description     TEXT NOT NULL,
    short_desc      VARCHAR(300),
    service_type    service_type NOT NULL,
    price_usd       NUMERIC(10,2) NOT NULL,
    duration_hours  NUMERIC(4,1),
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bookings (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    service_id          UUID NOT NULL REFERENCES service_listings(id),
    client_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scheduled_at        TIMESTAMPTZ NOT NULL,
    duration_minutes    INT NOT NULL DEFAULT 60,
    status              booking_status NOT NULL DEFAULT 'pending',
    zoom_meeting_id     TEXT,
    zoom_join_url       TEXT,
    zoom_start_url      TEXT,
    client_notes        TEXT,
    consultant_notes    TEXT,
    amount_paid_usd     NUMERIC(10,2),
    stripe_payment_id   TEXT,
    cancelled_reason    TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE client_projects (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           VARCHAR(200) NOT NULL,
    description     TEXT,
    service_type    service_type NOT NULL,
    status          project_status NOT NULL DEFAULT 'inquiry',
    budget_usd      NUMERIC(12,2),
    start_date      DATE,
    end_date        DATE,
    github_repo_url TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE availability_slots (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    day_of_week SMALLINT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    start_time  TIME NOT NULL,
    end_time    TIME NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bookings_client    ON bookings(client_id);
CREATE INDEX idx_bookings_scheduled ON bookings(scheduled_at);
CREATE INDEX idx_bookings_status    ON bookings(status);
CREATE INDEX idx_projects_client    ON client_projects(client_id);
CREATE INDEX idx_projects_status    ON client_projects(status);

CREATE TRIGGER service_listings_updated_at
    BEFORE UPDATE ON service_listings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER bookings_updated_at
    BEFORE UPDATE ON bookings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER client_projects_updated_at
    BEFORE UPDATE ON client_projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();