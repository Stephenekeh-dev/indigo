CREATE TYPE blockchain_network AS ENUM (
    'solana', 'polkadot', 'near', 'ethereum', 'substrate', 'other'
);
CREATE TYPE blockchain_project_type AS ENUM (
    'smart_contract', 'dapp', 'defi', 'nft', 'protocol', 'audit', 'consulting'
);
CREATE TYPE blockchain_project_status AS ENUM (
    'inquiry', 'scoping', 'in_progress', 'review', 'completed', 'cancelled'
);

CREATE TABLE blockchain_services (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title           VARCHAR(200) NOT NULL,
    slug            VARCHAR(220) NOT NULL UNIQUE,
    description     TEXT NOT NULL,
    network         blockchain_network NOT NULL,
    project_type    blockchain_project_type NOT NULL,
    price_from_usd  NUMERIC(12,2),
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blockchain_projects (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service_id          UUID REFERENCES blockchain_services(id),
    title               VARCHAR(200) NOT NULL,
    description         TEXT,
    network             blockchain_network NOT NULL,
    project_type        blockchain_project_type NOT NULL,
    status              blockchain_project_status NOT NULL DEFAULT 'inquiry',
    budget_usd          NUMERIC(12,2),
    contract_address    TEXT,
    repo_url            TEXT,
    testnet_url         TEXT,
    mainnet_url         TEXT,
    start_date          DATE,
    end_date            DATE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blockchain_inquiries (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(150) NOT NULL,
    email           VARCHAR(255) NOT NULL,
    company         VARCHAR(150),
    network         blockchain_network,
    project_type    blockchain_project_type,
    description     TEXT NOT NULL,
    budget_range    VARCHAR(100),
    responded_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blockchain_projects_client  ON blockchain_projects(client_id);
CREATE INDEX idx_blockchain_projects_status  ON blockchain_projects(status);
CREATE INDEX idx_blockchain_projects_network ON blockchain_projects(network);

CREATE TRIGGER blockchain_services_updated_at
    BEFORE UPDATE ON blockchain_services
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER blockchain_projects_updated_at
    BEFORE UPDATE ON blockchain_projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();