-- ============================================================
-- Helados Sofis - Migración inicial completa
-- Stack: PostgreSQL + SQLx
-- ============================================================

-- ─── USUARIOS Y AUTH ────────────────────────────────────

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL DEFAULT '',
    photo_url TEXT,
    role VARCHAR(20) NOT NULL CHECK (role IN ('owner', 'admin')),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    last_login TIMESTAMPTZ
);

-- ─── CATÁLOGO ───────────────────────────────────────────

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    modified_at TIMESTAMPTZ,
    modified_by UUID REFERENCES users(id)
);

CREATE TABLE flavors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    contact_info TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE workers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    phone VARCHAR(50),
    address TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    current_debt DECIMAL(12,2) NOT NULL DEFAULT 0,
    total_sales INTEGER NOT NULL DEFAULT 0,
    last_sale TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE freezers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    number INTEGER NOT NULL UNIQUE,
    max_capacity JSONB NOT NULL DEFAULT '{}',
    is_on BOOLEAN NOT NULL DEFAULT TRUE,
    last_toggle TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

-- ─── HISTORIAL DE PRECIOS ───────────────────────────────

CREATE TABLE price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    provider_id UUID NOT NULL REFERENCES providers(id),
    cost_price DECIMAL(10,2) NOT NULL,
    price_base DECIMAL(10,2) NOT NULL,
    price_route DECIMAL(10,2) NOT NULL,
    price_local DECIMAL(10,2) NOT NULL,
    commission DECIMAL(10,2) NOT NULL,
    effective_date TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_price_history_lookup ON price_history(
    product_id, flavor_id, provider_id, effective_date DESC
);
CREATE INDEX idx_price_history_effective ON price_history(effective_date);

-- ─── INVENTARIO ─────────────────────────────────────────

CREATE TABLE inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    freezer_id UUID NOT NULL REFERENCES freezers(id),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    provider_id UUID NOT NULL REFERENCES providers(id),
    quantity INTEGER NOT NULL CHECK (quantity >= 0),
    min_stock_alert INTEGER NOT NULL DEFAULT 20,
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE,
    assigned_worker_id UUID REFERENCES workers(id),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    CONSTRAINT unique_normal_inventory UNIQUE NULLS NOT DISTINCT (
        freezer_id, product_id, flavor_id, provider_id, is_deformed, assigned_worker_id
    )
);

CREATE INDEX idx_inventory_freezer ON inventory(freezer_id);
CREATE INDEX idx_inventory_product ON inventory(product_id, flavor_id);
CREATE INDEX idx_inventory_deformed ON inventory(is_deformed, assigned_worker_id);
CREATE INDEX idx_inventory_low_stock ON inventory(quantity) WHERE is_deformed = FALSE;

-- ─── COMPRAS A PROVEEDORES ──────────────────────────────

CREATE TABLE purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES providers(id),
    total DECIMAL(12,2) NOT NULL,
    payment_status VARCHAR(20) NOT NULL CHECK (payment_status IN ('paid', 'credit')),
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE purchase_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    purchase_id UUID NOT NULL REFERENCES purchases(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL,
    freezer_id UUID NOT NULL REFERENCES freezers(id)
);

-- ─── VIAJES DE TRABAJADORES ─────────────────────────────

CREATE TABLE worker_trips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL REFERENCES workers(id),
    departure_time TIMESTAMPTZ NOT NULL,
    return_time TIMESTAMPTZ,
    route_id UUID REFERENCES routes(id),
    status VARCHAR(20) NOT NULL CHECK (status IN ('in_progress', 'returned')),
    sold_quantity INTEGER NOT NULL DEFAULT 0,
    amount_due DECIMAL(12,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_worker_trips_worker ON worker_trips(worker_id);
CREATE INDEX idx_worker_trips_status ON worker_trips(status);
CREATE INDEX idx_worker_trips_departure ON worker_trips(departure_time);

CREATE TABLE worker_trip_loaded_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id UUID NOT NULL REFERENCES worker_trips(id) ON DELETE CASCADE,
    inventory_id UUID NOT NULL REFERENCES inventory(id),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    freezer_id UUID NOT NULL REFERENCES freezers(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL,
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_loaded_items_trip ON worker_trip_loaded_items(trip_id);

CREATE TABLE worker_trip_returned_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id UUID NOT NULL REFERENCES worker_trips(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE,
    destination_freezer_id UUID NOT NULL REFERENCES freezers(id)
);

CREATE INDEX idx_returned_items_trip ON worker_trip_returned_items(trip_id);

-- ─── PAGOS DE TRABAJADORES ──────────────────────────────

CREATE TABLE worker_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL REFERENCES workers(id),
    trip_id UUID REFERENCES worker_trips(id),
    amount DECIMAL(12,2) NOT NULL,
    previous_debt DECIMAL(12,2) NOT NULL,
    new_debt DECIMAL(12,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_worker_payments_worker ON worker_payments(worker_id);

-- ─── VENTAS LOCALES ─────────────────────────────────────

CREATE TABLE local_sales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    total DECIMAL(12,2) NOT NULL,
    sale_type VARCHAR(20) NOT NULL CHECK (sale_type IN ('local', 'custom', 'gift', 'family')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE local_sale_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sale_id UUID NOT NULL REFERENCES local_sales(id) ON DELETE CASCADE,
    inventory_id UUID NOT NULL REFERENCES inventory(id),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    freezer_id UUID NOT NULL REFERENCES freezers(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL
);

-- ─── VENTAS DEL DUEÑO ──────────────────────────────────

CREATE TABLE owner_sales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    departure_time TIMESTAMPTZ NOT NULL,
    return_time TIMESTAMPTZ,
    route_id UUID REFERENCES routes(id),
    sold_quantity INTEGER NOT NULL DEFAULT 0,
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    auto_withdrawal DECIMAL(12,2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE owner_sale_loaded_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sale_id UUID NOT NULL REFERENCES owner_sales(id) ON DELETE CASCADE,
    inventory_id UUID NOT NULL REFERENCES inventory(id),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    freezer_id UUID NOT NULL REFERENCES freezers(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL,
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE owner_sale_returned_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sale_id UUID NOT NULL REFERENCES owner_sales(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE,
    destination_freezer_id UUID NOT NULL REFERENCES freezers(id)
);

-- ─── CAJA REGISTRADORA (Event Sourcing) ─────────────────

CREATE TABLE cash_register (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(30) NOT NULL CHECK (type IN (
        'worker_payment', 'local_sale', 'owner_sale',
        'owner_withdrawal', 'expense'
    )),
    amount DECIMAL(12,2) NOT NULL,
    balance DECIMAL(12,2) NOT NULL,
    description TEXT,
    category VARCHAR(50),
    related_doc_type VARCHAR(50),
    related_doc_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_cash_register_created ON cash_register(created_at DESC);
CREATE INDEX idx_cash_register_type ON cash_register(type);

-- ─── TRANSFERENCIAS ENTRE CONGELADORES ──────────────────

CREATE TABLE freezer_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_freezer_id UUID NOT NULL REFERENCES freezers(id),
    to_freezer_id UUID NOT NULL REFERENCES freezers(id),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE freezer_transfer_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_id UUID NOT NULL REFERENCES freezer_transfers(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0)
);

-- ─── AUDITORÍA ──────────────────────────────────────────

CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(20) NOT NULL CHECK (action IN ('create', 'update', 'delete')),
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    changes_before JSONB,
    changes_after JSONB,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_table ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_user ON audit_log(created_by, created_at DESC);
