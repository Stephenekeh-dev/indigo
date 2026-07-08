CREATE TYPE event_type AS ENUM (
    'zoom_meetup', 'workshop', 'office_hours', 'conference', 'hackathon', 'webinar'
);
CREATE TYPE event_status AS ENUM ('scheduled', 'live', 'completed', 'cancelled');
CREATE TYPE membership_tier AS ENUM ('free', 'pro', 'enterprise');
CREATE TYPE membership_status AS ENUM ('active', 'cancelled', 'past_due', 'trialing');

CREATE TABLE events (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title               VARCHAR(200) NOT NULL,
    slug                VARCHAR(220) NOT NULL UNIQUE,
    description         TEXT NOT NULL,
    event_type          event_type NOT NULL,
    status              event_status NOT NULL DEFAULT 'scheduled',
    is_online           BOOLEAN NOT NULL DEFAULT TRUE,
    is_free             BOOLEAN NOT NULL DEFAULT TRUE,
    price_usd           NUMERIC(10,2) DEFAULT 0,
    max_attendees       INT,
    scheduled_at        TIMESTAMPTZ NOT NULL,
    duration_minutes    INT NOT NULL DEFAULT 60,
    timezone            VARCHAR(60) NOT NULL DEFAULT 'UTC',
    location            TEXT,
    zoom_meeting_id     TEXT,
    zoom_join_url       TEXT,
    zoom_start_url      TEXT,
    recording_url       TEXT,
    thumbnail_url       TEXT,
    tags                TEXT[] DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE event_registrations (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id            UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_id   TEXT,
    amount_paid_usd     NUMERIC(10,2),
    attended            BOOLEAN NOT NULL DEFAULT FALSE,
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_id, user_id)
);

CREATE TABLE memberships (
    id                      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tier                    membership_tier NOT NULL DEFAULT 'free',
    status                  membership_status NOT NULL DEFAULT 'active',
    stripe_subscription_id  TEXT UNIQUE,
    stripe_customer_id      TEXT,
    current_period_start    TIMESTAMPTZ,
    current_period_end      TIMESTAMPTZ,
    cancelled_at            TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE INDEX idx_events_scheduled    ON events(scheduled_at);
CREATE INDEX idx_events_status       ON events(status);
CREATE INDEX idx_registrations_user  ON event_registrations(user_id);
CREATE INDEX idx_memberships_user    ON memberships(user_id);
CREATE INDEX idx_memberships_status  ON memberships(status);

CREATE TRIGGER events_updated_at
    BEFORE UPDATE ON events
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER memberships_updated_at
    BEFORE UPDATE ON memberships
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();