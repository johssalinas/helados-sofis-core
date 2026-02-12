# Explicación Detallada del Modelo de Datos - Helados Sofis

Este documento complementa el plan principal con explicaciones PROFUNDAS de las tablas más complejas del sistema. Léelo ANTES de implementar para entender la lógica de negocio completa.

**Stack Técnico:** Backend en Rust (Axum) + PostgreSQL + SQLx

## Índice

1. [worker_trips - El Corazón del Sistema](#workertrips)
2. [cash_register - Event Sourcing](#cashregister)
3. [price_history - Temporal Data Pattern](#pricehistory)
4. [inventory - Granularidad Total](#inventory)
5. [owner_sales - Caso Especial del Dueño](#ownersales)

---

## worker_trips - El Corazón del Sistema {#workertrips}

**Por qué es la MÁS importante:** Registra TODO el ciclo de vida de una venta por trabajador, desde que carga hasta que paga.

### Estructura de Tablas (PostgreSQL)

```sql
-- Tabla principal de viajes
CREATE TABLE worker_trips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL REFERENCES workers(id),
    departure_time TIMESTAMPTZ NOT NULL,
    return_time TIMESTAMPTZ,  -- NULL si aún no regresa
    route_id UUID REFERENCES routes(id),
    status VARCHAR(20) NOT NULL CHECK (status IN ('in_progress', 'returned')),
    
    -- Campos calculados (denormalizados para performance)
    sold_quantity INTEGER NOT NULL DEFAULT 0,
    amount_due DECIMAL(12,2) NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

-- Items cargados en el viaje
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

-- Items devueltos del viaje
CREATE TABLE worker_trip_returned_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trip_id UUID NOT NULL REFERENCES worker_trips(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE,
    destination_freezer_id UUID NOT NULL REFERENCES freezers(id)
);

-- Índices para queries frecuentes
CREATE INDEX idx_worker_trips_worker ON worker_trips(worker_id);
CREATE INDEX idx_worker_trips_status ON worker_trips(status);
CREATE INDEX idx_worker_trips_departure ON worker_trips(departure_time);
CREATE INDEX idx_loaded_items_trip ON worker_trip_loaded_items(trip_id);
CREATE INDEX idx_returned_items_trip ON worker_trip_returned_items(trip_id);
```

### Modelos en Rust

```rust
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkerTrip {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub departure_time: DateTime<Utc>,
    pub return_time: Option<DateTime<Utc>>,
    pub route_id: Option<Uuid>,
    pub status: String,
    pub sold_quantity: i32,
    pub amount_due: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoadedItem {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub inventory_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub freezer_id: Uuid,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub is_deformed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReturnedItem {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub quantity: i32,
    pub is_deformed: bool,
    pub destination_freezer_id: Uuid,
}
```

### Ciclo de Vida Completo

#### FASE 1: Creación (Trabajador Carga)

**Momento:** Admin registra en sistema DESPUÉS de entregar físicamente los helados al trabajador.

**Ejemplo de datos:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "worker_id": "550e8400-e29b-41d4-a716-446655440010",
  "departure_time": "2025-11-18T08:30:00Z",
  "return_time": null,
  "route_id": "550e8400-e29b-41d4-a716-446655440020",
  "status": "in_progress",
  
  "loaded_items": [
    {
      "inventory_id": "inv-123",
      "product_id": "prod-paleta",
      "flavor_id": "flav-fresa",
      "freezer_id": "freezer-1",
      "quantity": 50,
      "unit_price": 1400,
      "is_deformed": false
    },
    {
      "inventory_id": "inv-456",
      "product_id": "prod-cono",
      "flavor_id": "flav-chocolate",
      "freezer_id": "freezer-2",
      "quantity": 30,
      "unit_price": 1600,
      "is_deformed": false
    },
    {
      "inventory_id": "inv-789",
      "product_id": "prod-paleta",
      "flavor_id": "flav-mora",
      "freezer_id": "freezer-3",
      "quantity": 5,
      "unit_price": 1400,
      "is_deformed": true
    }
  ],
  
  "returned_items": [],
  "sold_quantity": 0,
  "amount_due": 0
}
```

**Consecuencias AUTOMÁTICAS al crear (TRANSACCIÓN SQL):**

```rust
use sqlx::{PgPool, Transaction, Postgres};

pub async fn create_worker_trip(
    pool: &PgPool,
    trip_data: CreateTripRequest,
    created_by: Uuid,
) -> Result<WorkerTrip, AppError> {
    // Iniciar transacción para atomicidad
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
    
    // 1. Crear el viaje
    let trip = sqlx::query_as::<_, WorkerTrip>(
        r#"
        INSERT INTO worker_trips (worker_id, departure_time, route_id, status, created_by)
        VALUES ($1, $2, $3, 'in_progress', $4)
        RETURNING *
        "#
    )
    .bind(&trip_data.worker_id)
    .bind(&trip_data.departure_time)
    .bind(&trip_data.route_id)
    .bind(&created_by)
    .fetch_one(&mut *tx)
    .await?;
    
    // 2. Insertar items cargados y actualizar inventario
    for item in &trip_data.loaded_items {
        // Insertar item cargado
        sqlx::query(
            r#"
            INSERT INTO worker_trip_loaded_items 
            (trip_id, inventory_id, product_id, flavor_id, freezer_id, quantity, unit_price, is_deformed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(&trip.id)
        .bind(&item.inventory_id)
        .bind(&item.product_id)
        .bind(&item.flavor_id)
        .bind(&item.freezer_id)
        .bind(&item.quantity)
        .bind(&item.unit_price)
        .bind(&item.is_deformed)
        .execute(&mut *tx)
        .await?;
        
        // Actualizar inventario (restar cantidad)
        let rows_affected = sqlx::query(
            r#"
            UPDATE inventory 
            SET quantity = quantity - $1,
                last_updated = NOW(),
                updated_by = $2
            WHERE id = $3 AND quantity >= $1
            "#
        )
        .bind(&item.quantity)
        .bind(&created_by)
        .bind(&item.inventory_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        
        if rows_affected == 0 {
            return Err(AppError::InsufficientStock(item.inventory_id));
        }
        
        // Si era deformado y quedó en 0, eliminar el registro
        sqlx::query(
            r#"
            DELETE FROM inventory 
            WHERE id = $1 AND quantity = 0 AND is_deformed = true
            "#
        )
        .bind(&item.inventory_id)
        .execute(&mut *tx)
        .await?;
    }
    
    // 3. Crear registro de auditoría
    sqlx::query(
        r#"
        INSERT INTO audit_log (action, table_name, record_id, changes_after, created_by)
        VALUES ('create', 'worker_trips', $1, $2, $3)
        "#
    )
    .bind(&trip.id)
    .bind(&serde_json::to_value(&trip)?)
    .bind(&created_by)
    .execute(&mut *tx)
    .await?;
    
    // Confirmar transacción
    tx.commit().await?;
    
    Ok(trip)
}
```

#### FASE 2: Actualización (Trabajador Regresa)

**Momento:** Admin registra DESPUÉS de contar físicamente los helados devueltos e inspeccionar cuáles están deformados.

**Cálculos automáticos:**
- Paletas fresa: 50 - 10 = 40 vendidas
- Conos chocolate: 30 - 5 = 25 vendidos
- Paletas mora: 5 - 2 = 3 vendidas
- `sold_quantity`: 68 (40 + 25 + 3)
- `amount_due`: 100,200 ((40 + 3) × 1400 + 25 × 1600)

**Implementación en Rust:**

```rust
pub async fn complete_worker_trip(
    pool: &PgPool,
    trip_id: Uuid,
    returned_items: Vec<ReturnedItemRequest>,
    created_by: Uuid,
) -> Result<WorkerTrip, AppError> {
    let mut tx = pool.begin().await?;
    
    // 1. Obtener items cargados para calcular ventas
    let loaded_items = sqlx::query_as::<_, LoadedItem>(
        "SELECT * FROM worker_trip_loaded_items WHERE trip_id = $1"
    )
    .bind(&trip_id)
    .fetch_all(&mut *tx)
    .await?;
    
    // 2. Insertar items devueltos y procesar inventario
    for returned in &returned_items {
        // Insertar item devuelto
        sqlx::query(
            r#"
            INSERT INTO worker_trip_returned_items 
            (trip_id, product_id, flavor_id, quantity, is_deformed, destination_freezer_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(&trip_id)
        .bind(&returned.product_id)
        .bind(&returned.flavor_id)
        .bind(&returned.quantity)
        .bind(&returned.is_deformed)
        .bind(&returned.destination_freezer_id)
        .execute(&mut *tx)
        .await?;
        
        if returned.is_deformed {
            // ═══ DEFORMADOS → Crear NUEVO item asignado al trabajador ═══
            let worker_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT worker_id FROM worker_trips WHERE id = $1"
            )
            .bind(&trip_id)
            .fetch_one(&mut *tx)
            .await?;
            
            // Obtener provider_id del item original
            let provider_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT i.provider_id FROM inventory i
                JOIN worker_trip_loaded_items li ON i.id = li.inventory_id
                WHERE li.trip_id = $1 AND li.product_id = $2 AND li.flavor_id = $3
                LIMIT 1
                "#
            )
            .bind(&trip_id)
            .bind(&returned.product_id)
            .bind(&returned.flavor_id)
            .fetch_one(&mut *tx)
            .await?;
            
            sqlx::query(
                r#"
                INSERT INTO inventory 
                (freezer_id, product_id, flavor_id, provider_id, quantity, is_deformed, 
                 assigned_worker_id, min_stock_alert, updated_by)
                VALUES ($1, $2, $3, $4, $5, true, $6, 0, $7)
                "#
            )
            .bind(&returned.destination_freezer_id)
            .bind(&returned.product_id)
            .bind(&returned.flavor_id)
            .bind(&provider_id)
            .bind(&returned.quantity)
            .bind(&worker_id)
            .bind(&created_by)
            .execute(&mut *tx)
            .await?;
            
        } else {
            // ═══ BUENOS → Sumar a inventario existente o crear nuevo ═══
            let provider_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT i.provider_id FROM inventory i
                JOIN worker_trip_loaded_items li ON i.id = li.inventory_id
                WHERE li.trip_id = $1 AND li.product_id = $2 AND li.flavor_id = $3
                LIMIT 1
                "#
            )
            .bind(&trip_id)
            .bind(&returned.product_id)
            .bind(&returned.flavor_id)
            .fetch_one(&mut *tx)
            .await?;
            
            // Intentar actualizar existente con UPSERT
            sqlx::query(
                r#"
                INSERT INTO inventory 
                (freezer_id, product_id, flavor_id, provider_id, quantity, is_deformed, min_stock_alert, updated_by)
                VALUES ($1, $2, $3, $4, $5, false, 20, $6)
                ON CONFLICT (freezer_id, product_id, flavor_id, provider_id, is_deformed) 
                WHERE is_deformed = false AND assigned_worker_id IS NULL
                DO UPDATE SET 
                    quantity = inventory.quantity + EXCLUDED.quantity,
                    last_updated = NOW(),
                    updated_by = EXCLUDED.updated_by
                "#
            )
            .bind(&returned.destination_freezer_id)
            .bind(&returned.product_id)
            .bind(&returned.flavor_id)
            .bind(&provider_id)
            .bind(&returned.quantity)
            .bind(&created_by)
            .execute(&mut *tx)
            .await?;
        }
    }
    
    // 3. Calcular sold_quantity y amount_due
    let (sold_quantity, amount_due) = calculate_sales(&loaded_items, &returned_items);
    
    // 4. Actualizar el viaje
    let trip = sqlx::query_as::<_, WorkerTrip>(
        r#"
        UPDATE worker_trips 
        SET return_time = NOW(),
            status = 'returned',
            sold_quantity = $1,
            amount_due = $2
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&sold_quantity)
    .bind(&amount_due)
    .bind(&trip_id)
    .fetch_one(&mut *tx)
    .await?;
    
    // 5. Actualizar deuda del trabajador (denormalizado)
    sqlx::query(
        r#"
        UPDATE workers 
        SET current_debt = current_debt + $1,
            total_sales = total_sales + $2,
            last_sale = NOW()
        WHERE id = $3
        "#
    )
    .bind(&amount_due)
    .bind(&sold_quantity)
    .bind(&trip.worker_id)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok(trip)
}

fn calculate_sales(
    loaded: &[LoadedItem], 
    returned: &[ReturnedItemRequest]
) -> (i32, rust_decimal::Decimal) {
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    
    // Agrupar cantidades devueltas por producto+sabor
    let mut returned_map: HashMap<(Uuid, Uuid), i32> = HashMap::new();
    for r in returned {
        *returned_map.entry((r.product_id, r.flavor_id)).or_insert(0) += r.quantity;
    }
    
    let mut total_sold = 0i32;
    let mut total_amount = Decimal::ZERO;
    
    for item in loaded {
        let returned_qty = returned_map
            .get(&(item.product_id, item.flavor_id))
            .copied()
            .unwrap_or(0);
        
        let sold = item.quantity - returned_qty;
        total_sold += sold;
        total_amount += item.unit_price * Decimal::from(sold);
    }
    
    (total_sold, total_amount)
}
```

**Si trabajador paga INMEDIATAMENTE:**

```rust
pub async fn process_immediate_payment(
    pool: &PgPool,
    trip_id: Uuid,
    created_by: Uuid,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    
    // Obtener el viaje y datos del trabajador
    let trip = sqlx::query_as::<_, WorkerTrip>(
        "SELECT * FROM worker_trips WHERE id = $1"
    )
    .bind(&trip_id)
    .fetch_one(&mut *tx)
    .await?;
    
    let worker = sqlx::query_as::<_, Worker>(
        "SELECT * FROM workers WHERE id = $1"
    )
    .bind(&trip.worker_id)
    .fetch_one(&mut *tx)
    .await?;
    
    // 1. Registrar pago
    let payment_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO worker_payments 
        (worker_id, trip_id, amount, previous_debt, new_debt, created_by)
        VALUES ($1, $2, $3, $4, 0, $5)
        RETURNING id
        "#
    )
    .bind(&trip.worker_id)
    .bind(&trip_id)
    .bind(&trip.amount_due)
    .bind(&worker.current_debt)
    .bind(&created_by)
    .fetch_one(&mut *tx)
    .await?;
    
    // 2. Actualizar deuda del trabajador a 0
    sqlx::query("UPDATE workers SET current_debt = 0 WHERE id = $1")
        .bind(&trip.worker_id)
        .execute(&mut *tx)
        .await?;
    
    // 3. Registrar en caja con Event Sourcing
    let current_balance = get_current_balance_tx(&mut tx).await?;
    let new_balance = current_balance + trip.amount_due;
    
    sqlx::query(
        r#"
        INSERT INTO cash_register 
        (type, amount, balance, related_doc_type, related_doc_id, created_by)
        VALUES ('worker_payment', $1, $2, 'worker_payments', $3, $4)
        "#
    )
    .bind(&trip.amount_due)
    .bind(&new_balance)
    .bind(&payment_id)
    .bind(&created_by)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok(())
}
```

### Por Qué Esta Estructura

**`loaded_items` es tabla separada (1:N):**
- Un trabajador carga de MÚLTIPLES congeladores en UNA salida
- Ejemplo: 50 paletas del congelador 1 + 30 conos del congelador 2
- Necesitamos saber de QUÉ congelador salió cada producto
- Para reportes: "El congelador 1 tiene rotación más alta que el 2"

**`inventory_id` en loaded_items:**
- Para saber EXACTAMENTE qué item del inventario se sacó
- Crítico si ese item era deformado asignado al trabajador
- Permite revertir operación si hay error

**`returned_items` es tabla separada (no columna JSON):**
- No todos los productos cargados se devuelven (mayoría se vende)
- Admin registra solo lo devuelto, no tiene que buscar en loaded_items
- Facilita queries y reportes con SQL estándar

**`is_deformed` está en `returned_items`, no en `loaded_items`:**
- Al CARGAR: helados están bien congelados
- En la RUTA: pueden derretirse parcialmente
- Al REGRESAR: admin inspecciona visualmente y MARCA deformados
- **Excepción:** `loaded_items.is_deformed = true` si el trabajador cargó helados que YA eran deformados (asignados a él previamente)

**Campos calculados (`sold_quantity`, `amount_due`) denormalizados:**
- Evita calcular cada vez que se consulta el trip
- Queries rápidas: "ventas totales del mes" suma `sold_quantity`
- Si hay bug en cálculo, se puede recalcular desde las tablas relacionadas

### Queries Importantes (SQL + Rust)

```rust
// 1. Viajes activos (trabajadores que aún no regresan)
pub async fn get_active_trips(pool: &PgPool) -> Result<Vec<WorkerTrip>, AppError> {
    sqlx::query_as::<_, WorkerTrip>(
        r#"
        SELECT * FROM worker_trips 
        WHERE status = 'in_progress'
        ORDER BY departure_time DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 2. Historial de un trabajador
pub async fn get_worker_history(
    pool: &PgPool, 
    worker_id: Uuid,
    limit: i64,
) -> Result<Vec<WorkerTrip>, AppError> {
    sqlx::query_as::<_, WorkerTrip>(
        r#"
        SELECT * FROM worker_trips 
        WHERE worker_id = $1
        ORDER BY departure_time DESC
        LIMIT $2
        "#
    )
    .bind(&worker_id)
    .bind(&limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 3. Ventas de hoy
pub async fn get_todays_sales(pool: &PgPool) -> Result<Vec<WorkerTrip>, AppError> {
    sqlx::query_as::<_, WorkerTrip>(
        r#"
        SELECT * FROM worker_trips 
        WHERE departure_time >= CURRENT_DATE
          AND departure_time < CURRENT_DATE + INTERVAL '1 day'
          AND status = 'returned'
        ORDER BY departure_time DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 4. Ventas de un periodo (para reportes)
pub async fn get_sales_in_range(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<WorkerTrip>, AppError> {
    sqlx::query_as::<_, WorkerTrip>(
        r#"
        SELECT * FROM worker_trips 
        WHERE departure_time >= $1
          AND departure_time < $2
          AND status = 'returned'
        ORDER BY departure_time DESC
        "#
    )
    .bind(&from)
    .bind(&to)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 5. Viaje con todos sus items (JOIN)
pub async fn get_trip_with_items(
    pool: &PgPool,
    trip_id: Uuid,
) -> Result<TripWithItems, AppError> {
    let trip = sqlx::query_as::<_, WorkerTrip>(
        "SELECT * FROM worker_trips WHERE id = $1"
    )
    .bind(&trip_id)
    .fetch_one(pool)
    .await?;
    
    let loaded_items = sqlx::query_as::<_, LoadedItem>(
        "SELECT * FROM worker_trip_loaded_items WHERE trip_id = $1"
    )
    .bind(&trip_id)
    .fetch_all(pool)
    .await?;
    
    let returned_items = sqlx::query_as::<_, ReturnedItem>(
        "SELECT * FROM worker_trip_returned_items WHERE trip_id = $1"
    )
    .bind(&trip_id)
    .fetch_all(pool)
    .await?;
    
    Ok(TripWithItems { trip, loaded_items, returned_items })
}
```

---

## cash_register - Event Sourcing para Trazabilidad Total {#cashregister}

### Concepto: Event Sourcing

**NO guardamos:** "Balance actual = $150,000"

**SÍ guardamos:** TODOS los eventos que llevaron a ese balance:
- +$50,000 (pago de Juan)
- +$30,000 (venta local)
- -$20,000 (gasto de luz)
- -$10,000 (retiro del dueño)
- = $150,000

### Estructura SQL

```sql
CREATE TABLE cash_register (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(30) NOT NULL CHECK (type IN (
        'worker_payment', 'local_sale', 'owner_sale', 
        'owner_withdrawal', 'expense'
    )),
    amount DECIMAL(12,2) NOT NULL,  -- + ingreso, - egreso
    balance DECIMAL(12,2) NOT NULL,  -- Balance DESPUÉS de este evento
    description TEXT,
    category VARCHAR(50),  -- Solo para type='expense'
    related_doc_type VARCHAR(50),  -- 'worker_payments', 'local_sales', etc.
    related_doc_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

-- Índice para obtener último balance rápidamente
CREATE INDEX idx_cash_register_created ON cash_register(created_at DESC);
CREATE INDEX idx_cash_register_type ON cash_register(type);

-- Vista materializada para balance actual (opcional, para performance extrema)
CREATE MATERIALIZED VIEW current_balance AS
SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1;
```

### Modelo en Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashTransaction {
    pub id: Uuid,
    pub r#type: String,
    pub amount: Decimal,
    pub balance: Decimal,
    pub description: Option<String>,
    pub category: Option<String>,
    pub related_doc_type: Option<String>,
    pub related_doc_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CashTransactionType {
    WorkerPayment,
    LocalSale,
    OwnerSale,
    OwnerWithdrawal,
    Expense,
}
```

### Tipos de Eventos (type)

1. **`worker_payment`** - Trabajador pagó su deuda
   - `amount`: Positivo (es ingreso)
   - `related_doc_id`: UUID de `worker_payments`

2. **`local_sale`** - Venta en el local
   - `amount`: Positivo
   - `related_doc_id`: UUID de `local_sales`

3. **`owner_sale`** - Venta del dueño (INGRESO momentáneo)
   - `amount`: Positivo
   - `related_doc_id`: UUID de `owner_sales`
   - **IMPORTANTE:** Siempre seguido INMEDIATAMENTE de `owner_withdrawal`

4. **`owner_withdrawal`** - Retiro del dueño
   - `amount`: Negativo
   - `related_doc_id`: UUID de `owner_sales` (si auto) o NULL (si manual)

5. **`expense`** - Gasto operativo
   - `amount`: Negativo
   - `category`: "luz" | "agua" | "mantenimiento" | "transporte" | "otros"
   - Solo el DUEÑO puede crear

### Por Qué Event Sourcing

1. **Auditoría completa:** Puedes ver CADA movimiento históricamente
2. **Reconstrucción del balance:** Si está corrupto, recalculas sumando todos los `amount`
3. **Reportes:** Filtrar por tipo y sumar es trivial con SQL
4. **Debugging:** Rastreas exactamente qué transacción causó un problema
5. **Inmutabilidad:** Nunca modificas eventos pasados, si hay error creas evento de corrección

### Cálculo de Balance Actual

```rust
// Opción 1: RÁPIDA (producción) - O(1)
pub async fn get_current_balance(pool: &PgPool) -> Result<Decimal, AppError> {
    sqlx::query_scalar::<_, Option<Decimal>>(
        r#"
        SELECT balance FROM cash_register 
        ORDER BY created_at DESC 
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await
    .map(|b| b.unwrap_or(Decimal::ZERO))
    .map_err(AppError::from)
}

// Opción 2: LENTA pero PRECISA (auditoría) - O(n)
pub async fn calculate_balance_from_scratch(pool: &PgPool) -> Result<Decimal, AppError> {
    sqlx::query_scalar::<_, Option<Decimal>>(
        "SELECT COALESCE(SUM(amount), 0) FROM cash_register"
    )
    .fetch_one(pool)
    .await
    .map(|b| b.unwrap_or(Decimal::ZERO))
    .map_err(AppError::from)
}

// En auditoría mensual, comparas ambos:
pub async fn verify_balance_integrity(pool: &PgPool) -> Result<bool, AppError> {
    let denormalized = get_current_balance(pool).await?;
    let calculated = calculate_balance_from_scratch(pool).await?;
    
    if denormalized != calculated {
        tracing::error!(
            "Balance corrupto! Denormalizado: {}, Calculado: {}",
            denormalized, calculated
        );
        return Ok(false);
    }
    
    Ok(true)
}
```

### Crear Nuevo Evento (TRANSACCIÓN para atomicidad)

**CRÍTICO:** Usar transacción para evitar race conditions.

```rust
pub async fn add_cash_transaction(
    pool: &PgPool,
    transaction_type: CashTransactionType,
    amount: Decimal,
    description: Option<String>,
    category: Option<String>,
    related_doc_type: Option<String>,
    related_doc_id: Option<Uuid>,
    created_by: Uuid,
) -> Result<CashTransaction, AppError> {
    let mut tx = pool.begin().await?;
    
    // 1. Obtener balance actual CON LOCK para evitar race conditions
    let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
        r#"
        SELECT balance FROM cash_register 
        ORDER BY created_at DESC 
        LIMIT 1
        FOR UPDATE
        "#
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .unwrap_or(Decimal::ZERO);
    
    // 2. Calcular nuevo balance
    let new_balance = current_balance + amount;
    
    // 3. Insertar evento
    let transaction = sqlx::query_as::<_, CashTransaction>(
        r#"
        INSERT INTO cash_register 
        (type, amount, balance, description, category, related_doc_type, related_doc_id, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(&transaction_type.as_str())
    .bind(&amount)
    .bind(&new_balance)
    .bind(&description)
    .bind(&category)
    .bind(&related_doc_type)
    .bind(&related_doc_id)
    .bind(&created_by)
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok(transaction)
}

impl CashTransactionType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::WorkerPayment => "worker_payment",
            Self::LocalSale => "local_sale",
            Self::OwnerSale => "owner_sale",
            Self::OwnerWithdrawal => "owner_withdrawal",
            Self::Expense => "expense",
        }
    }
}
```

**Por qué transacción con FOR UPDATE:**
- Si 2 admins registran pagos SIMULTÁNEAMENTE
- Sin lock: ambos leen balance = $100,000
- Admin A registra +$50,000 → balance = $150,000
- Admin B registra +$30,000 → balance = $130,000 (INCORRECTO, debería ser $180,000)
- Con `FOR UPDATE`: uno espera al otro, balance correcto

### Queries de Reportes

```rust
// Ingresos del mes
pub async fn get_monthly_income(
    pool: &PgPool,
    year: i32,
    month: u32,
) -> Result<Decimal, AppError> {
    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let end = start + chrono::Months::new(1);
    
    sqlx::query_scalar::<_, Option<Decimal>>(
        r#"
        SELECT COALESCE(SUM(amount), 0) 
        FROM cash_register 
        WHERE created_at >= $1 
          AND created_at < $2 
          AND amount > 0
        "#
    )
    .bind(&start)
    .bind(&end)
    .fetch_one(pool)
    .await
    .map(|b| b.unwrap_or(Decimal::ZERO))
    .map_err(AppError::from)
}

// Gastos por categoría
pub async fn get_expenses_by_category(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<HashMap<String, Decimal>, AppError> {
    let rows = sqlx::query_as::<_, (String, Decimal)>(
        r#"
        SELECT category, ABS(SUM(amount)) as total
        FROM cash_register 
        WHERE type = 'expense'
          AND created_at >= $1 
          AND created_at < $2
        GROUP BY category
        "#
    )
    .bind(&from)
    .bind(&to)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().collect())
}

// Movimientos de hoy (para dashboard)
pub async fn get_todays_transactions(pool: &PgPool) -> Result<Vec<CashTransaction>, AppError> {
    sqlx::query_as::<_, CashTransaction>(
        r#"
        SELECT * FROM cash_register 
        WHERE created_at >= CURRENT_DATE
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
```

---

## price_history - Temporal Data Pattern {#pricehistory}

### El Problema

Los proveedores cambian precios frecuentemente. Necesitamos:
- Usar precio ACTUAL para nuevas ventas
- Usar precio HISTÓRICO para reportes de ventas pasadas (precisión contable)
- Saber "cuánto ha subido el precio en 6 meses"

### La Solución: Temporal Data

En vez de actualizar el precio existente, CREAMOS un nuevo registro con `effective_date`.

### Estructura SQL

```sql
CREATE TABLE price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    provider_id UUID NOT NULL REFERENCES providers(id),
    
    -- Los 4 precios del sistema
    cost_price DECIMAL(10,2) NOT NULL,   -- Lo que cuesta al negocio
    price_base DECIMAL(10,2) NOT NULL,   -- Lo que paga trabajador al negocio
    price_route DECIMAL(10,2) NOT NULL,  -- Lo que cobra trabajador a cliente
    price_local DECIMAL(10,2) NOT NULL,  -- Precio venta en local
    
    commission DECIMAL(10,2) NOT NULL,   -- price_route - price_base
    
    effective_date TIMESTAMPTZ NOT NULL, -- Desde cuándo aplica
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índice para query de precio actual (más reciente primero)
CREATE INDEX idx_price_history_lookup ON price_history(
    product_id, flavor_id, provider_id, effective_date DESC
);

-- Índice para query de precio en fecha específica
CREATE INDEX idx_price_history_effective ON price_history(effective_date);
```

### Modelo en Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriceHistory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub cost_price: Decimal,
    pub price_base: Decimal,
    pub price_route: Decimal,
    pub price_local: Decimal,
    pub commission: Decimal,
    pub effective_date: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
```

### Queries

**1. Obtener precio ACTUAL:**

```rust
pub async fn get_current_price(
    pool: &PgPool,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
) -> Result<PriceHistory, AppError> {
    sqlx::query_as::<_, PriceHistory>(
        r#"
        SELECT * FROM price_history 
        WHERE product_id = $1 
          AND flavor_id = $2 
          AND provider_id = $3
        ORDER BY effective_date DESC
        LIMIT 1
        "#
    )
    .bind(&product_id)
    .bind(&flavor_id)
    .bind(&provider_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Price not found".into()))
}
```

**2. Obtener precio en fecha ESPECÍFICA (para reportes):**

```rust
pub async fn get_price_at(
    pool: &PgPool,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
    date: DateTime<Utc>,
) -> Result<PriceHistory, AppError> {
    sqlx::query_as::<_, PriceHistory>(
        r#"
        SELECT * FROM price_history 
        WHERE product_id = $1 
          AND flavor_id = $2 
          AND provider_id = $3
          AND effective_date <= $4
        ORDER BY effective_date DESC
        LIMIT 1
        "#
    )
    .bind(&product_id)
    .bind(&flavor_id)
    .bind(&provider_id)
    .bind(&date)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("No price existed on that date".into()))
}
```

### Crear Nuevo Precio (NO actualizar)

```rust
pub async fn create_new_price(
    pool: &PgPool,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
    cost_price: Decimal,
    price_base: Decimal,
    price_route: Decimal,
    price_local: Decimal,
    created_by: Uuid,
) -> Result<PriceHistory, AppError> {
    let commission = price_route - price_base;
    
    // NO actualizamos el registro anterior
    // Creamos NUEVO registro con effective_date = AHORA
    sqlx::query_as::<_, PriceHistory>(
        r#"
        INSERT INTO price_history 
        (product_id, flavor_id, provider_id, cost_price, price_base, 
         price_route, price_local, commission, effective_date, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
        RETURNING *
        "#
    )
    .bind(&product_id)
    .bind(&flavor_id)
    .bind(&provider_id)
    .bind(&cost_price)
    .bind(&price_base)
    .bind(&price_route)
    .bind(&price_local)
    .bind(&commission)
    .bind(&created_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
    
    // El registro anterior queda intacto para historial
}
```

### Ejemplo de Reporte con Precios Históricos

```rust
// Reporte: "Ganancia neta de la primera semana de noviembre"
pub async fn calculate_net_profit(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Decimal, AppError> {
    // SQL con JOIN a price_history usando fecha del viaje
    let result = sqlx::query_scalar::<_, Option<Decimal>>(
        r#"
        SELECT SUM(
            (li.quantity - COALESCE(
                (SELECT SUM(ri.quantity) 
                 FROM worker_trip_returned_items ri 
                 WHERE ri.trip_id = wt.id 
                   AND ri.product_id = li.product_id 
                   AND ri.flavor_id = li.flavor_id
                ), 0
            )) * (ph.price_route - ph.cost_price)
        ) as net_profit
        FROM worker_trips wt
        JOIN worker_trip_loaded_items li ON li.trip_id = wt.id
        JOIN inventory inv ON inv.id = li.inventory_id
        JOIN LATERAL (
            SELECT * FROM price_history 
            WHERE product_id = li.product_id 
              AND flavor_id = li.flavor_id 
              AND provider_id = inv.provider_id
              AND effective_date <= wt.departure_time
            ORDER BY effective_date DESC
            LIMIT 1
        ) ph ON true
        WHERE wt.departure_time >= $1 
          AND wt.departure_time < $2
          AND wt.status = 'returned'
        "#
    )
    .bind(&from)
    .bind(&to)
    .fetch_one(pool)
    .await?;
    
    Ok(result.unwrap_or(Decimal::ZERO))
}
```

---

## inventory - Granularidad Total por Congelador {#inventory}

### Filosofía: Una "Pila" Homogénea por Fila

Cada fila de inventory representa una "pila" de helados IDÉNTICOS:
- Mismo congelador
- Mismo tipo de producto
- Mismo sabor
- Mismo proveedor
- Mismo estado (normal o deformado)

### Estructura SQL

```sql
CREATE TABLE inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    freezer_id UUID NOT NULL REFERENCES freezers(id),
    product_id UUID NOT NULL REFERENCES products(id),
    flavor_id UUID NOT NULL REFERENCES flavors(id),
    provider_id UUID NOT NULL REFERENCES providers(id),
    
    quantity INTEGER NOT NULL CHECK (quantity >= 0),
    min_stock_alert INTEGER NOT NULL DEFAULT 20,
    
    -- Estado especial para deformados
    is_deformed BOOLEAN NOT NULL DEFAULT FALSE,
    assigned_worker_id UUID REFERENCES workers(id),  -- Solo si is_deformed=true
    
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    
    -- Constraint para evitar duplicados de items "normales"
    CONSTRAINT unique_normal_inventory UNIQUE NULLS NOT DISTINCT (
        freezer_id, product_id, flavor_id, provider_id, is_deformed, assigned_worker_id
    )
);

-- Índices
CREATE INDEX idx_inventory_freezer ON inventory(freezer_id);
CREATE INDEX idx_inventory_product ON inventory(product_id, flavor_id);
CREATE INDEX idx_inventory_deformed ON inventory(is_deformed, assigned_worker_id);
CREATE INDEX idx_inventory_low_stock ON inventory(quantity) WHERE is_deformed = FALSE;
```

### Modelo en Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub freezer_id: Uuid,
    pub product_id: Uuid,
    pub flavor_id: Uuid,
    pub provider_id: Uuid,
    pub quantity: i32,
    pub min_stock_alert: i32,
    pub is_deformed: bool,
    pub assigned_worker_id: Option<Uuid>,
    pub last_updated: DateTime<Utc>,
    pub updated_by: Uuid,
}
```

### Por Qué Separar por Congelador

**Razón 1:** Dueño quiere saber cuántos hay EN CADA congelador
```rust
pub async fn get_freezer_total(pool: &PgPool, freezer_id: Uuid) -> Result<i32, AppError> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COALESCE(SUM(quantity), 0) FROM inventory WHERE freezer_id = $1"
    )
    .bind(&freezer_id)
    .fetch_one(pool)
    .await
    .map(|v| v.unwrap_or(0) as i32)
    .map_err(AppError::from)
}
```

**Razón 2:** Detectar desbalance entre congeladores
**Razón 3:** Consolidación de congeladores (apagar uno y mover stock)

### Por Qué Separar Deformados

```rust
// Stock disponible para venta (excluye deformados)
pub async fn get_sellable_stock(pool: &PgPool) -> Result<Vec<InventoryItem>, AppError> {
    sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE is_deformed = FALSE"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// Helados deformados asignados a un trabajador
pub async fn get_worker_deformed(
    pool: &PgPool, 
    worker_id: Uuid,
) -> Result<Vec<InventoryItem>, AppError> {
    sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT * FROM inventory 
        WHERE assigned_worker_id = $1 AND is_deformed = TRUE
        "#
    )
    .bind(&worker_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
```

### Operaciones CRUD

#### 1. Compra a Proveedor (AGREGAR stock)

```rust
pub async fn register_purchase(
    pool: &PgPool,
    freezer_id: Uuid,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
    quantity: i32,
    updated_by: Uuid,
) -> Result<InventoryItem, AppError> {
    // UPSERT: si existe incrementa, si no existe crea
    sqlx::query_as::<_, InventoryItem>(
        r#"
        INSERT INTO inventory 
        (freezer_id, product_id, flavor_id, provider_id, quantity, 
         is_deformed, min_stock_alert, updated_by)
        VALUES ($1, $2, $3, $4, $5, FALSE, 20, $6)
        ON CONFLICT (freezer_id, product_id, flavor_id, provider_id, is_deformed, assigned_worker_id) 
        WHERE is_deformed = FALSE AND assigned_worker_id IS NULL
        DO UPDATE SET 
            quantity = inventory.quantity + EXCLUDED.quantity,
            last_updated = NOW(),
            updated_by = EXCLUDED.updated_by
        RETURNING *
        "#
    )
    .bind(&freezer_id)
    .bind(&product_id)
    .bind(&flavor_id)
    .bind(&provider_id)
    .bind(&quantity)
    .bind(&updated_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}
```

#### 2. Trabajador Devuelve DEFORMADOS

```rust
pub async fn return_deformed(
    pool: &PgPool,
    destination_freezer_id: Uuid,
    product_id: Uuid,
    flavor_id: Uuid,
    provider_id: Uuid,
    worker_id: Uuid,
    quantity: i32,
    updated_by: Uuid,
) -> Result<InventoryItem, AppError> {
    // Siempre crear NUEVO item (no sumar a existentes)
    sqlx::query_as::<_, InventoryItem>(
        r#"
        INSERT INTO inventory 
        (freezer_id, product_id, flavor_id, provider_id, quantity, 
         is_deformed, assigned_worker_id, min_stock_alert, updated_by)
        VALUES ($1, $2, $3, $4, $5, TRUE, $6, 0, $7)
        RETURNING *
        "#
    )
    .bind(&destination_freezer_id)
    .bind(&product_id)
    .bind(&flavor_id)
    .bind(&provider_id)
    .bind(&quantity)
    .bind(&worker_id)
    .bind(&updated_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}
```

### Queries Útiles

```rust
// 1. Stock total de un producto (todos los congeladores)
pub async fn get_total_stock(
    pool: &PgPool,
    product_id: Uuid,
    flavor_id: Uuid,
) -> Result<i32, AppError> {
    sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COALESCE(SUM(quantity), 0) FROM inventory 
        WHERE product_id = $1 AND flavor_id = $2 AND is_deformed = FALSE
        "#
    )
    .bind(&product_id)
    .bind(&flavor_id)
    .fetch_one(pool)
    .await
    .map(|v| v.unwrap_or(0) as i32)
    .map_err(AppError::from)
}

// 2. Stock de un congelador
pub async fn get_freezer_stock(
    pool: &PgPool,
    freezer_id: Uuid,
) -> Result<Vec<InventoryItem>, AppError> {
    sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE freezer_id = $1"
    )
    .bind(&freezer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 3. Productos con stock bajo (alerta)
pub async fn get_low_stock_items(pool: &PgPool) -> Result<Vec<InventoryItem>, AppError> {
    sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT * FROM inventory 
        WHERE is_deformed = FALSE 
          AND quantity <= min_stock_alert
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

// 4. Capacidad disponible de un congelador
pub async fn get_available_capacity(
    pool: &PgPool,
    freezer_id: Uuid,
) -> Result<HashMap<Uuid, i32>, AppError> {
    // Obtener capacidad máxima del congelador
    let freezer = sqlx::query_as::<_, Freezer>(
        "SELECT * FROM freezers WHERE id = $1"
    )
    .bind(&freezer_id)
    .fetch_one(pool)
    .await?;
    
    // Obtener stock actual agrupado por producto
    let current_stock = sqlx::query_as::<_, (Uuid, i64)>(
        r#"
        SELECT product_id, SUM(quantity) as total
        FROM inventory 
        WHERE freezer_id = $1
        GROUP BY product_id
        "#
    )
    .bind(&freezer_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, qty)| (id, qty as i32))
    .collect::<HashMap<_, _>>();
    
    // Calcular disponible
    let mut available = HashMap::new();
    for (product_id, max_cap) in &freezer.max_capacity {
        let used = current_stock.get(product_id).copied().unwrap_or(0);
        available.insert(*product_id, max_cap - used);
    }
    
    Ok(available)
}
```

---

## owner_sales - Caso Especial del Dueño {#ownersales}

### El Problema

El dueño también vende helados en rutas, pero:
- NO paga comisión al negocio (porque ÉL ES el negocio)
- El dinero NO queda en la caja (se lo lleva inmediatamente)
- PERO necesitamos registrar sus ventas para reportes

### La Solución: Registro + Retiro Automático

### Estructura SQL

```sql
CREATE TABLE owner_sales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id),
    departure_time TIMESTAMPTZ NOT NULL,
    return_time TIMESTAMPTZ,
    route_id UUID REFERENCES routes(id),
    
    sold_quantity INTEGER NOT NULL DEFAULT 0,
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
    auto_withdrawal DECIMAL(12,2) NOT NULL DEFAULT 0,  -- = total_amount siempre
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id)
);

-- Tablas de items similares a worker_trips
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
```

### Diferencias Críticas con worker_trips

1. **NO se crea `worker_payments`** (el dueño no paga)
2. **NO se actualiza `workers.current_debt`** (siempre es 0)
3. **SÍ se crean 2 registros en `cash_register`:**

### Flujo Completo

```rust
pub async fn register_owner_sale(
    pool: &PgPool,
    sale_data: CreateOwnerSaleRequest,
    owner_id: Uuid,
) -> Result<OwnerSale, AppError> {
    let mut tx = pool.begin().await?;
    
    // 1. Crear owner_sale
    let sale = sqlx::query_as::<_, OwnerSale>(
        r#"
        INSERT INTO owner_sales (owner_id, departure_time, route_id, created_by)
        VALUES ($1, $2, $3, $1)
        RETURNING *
        "#
    )
    .bind(&owner_id)
    .bind(&sale_data.departure_time)
    .bind(&sale_data.route_id)
    .fetch_one(&mut *tx)
    .await?;
    
    // 2. Insertar loaded_items y actualizar inventario (igual que worker_trips)
    for item in &sale_data.loaded_items {
        sqlx::query(
            r#"
            INSERT INTO owner_sale_loaded_items 
            (sale_id, inventory_id, product_id, flavor_id, freezer_id, quantity, unit_price)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(&sale.id)
        .bind(&item.inventory_id)
        .bind(&item.product_id)
        .bind(&item.flavor_id)
        .bind(&item.freezer_id)
        .bind(&item.quantity)
        .bind(&item.unit_price)
        .execute(&mut *tx)
        .await?;
        
        // Restar del inventario
        sqlx::query(
            "UPDATE inventory SET quantity = quantity - $1 WHERE id = $2"
        )
        .bind(&item.quantity)
        .bind(&item.inventory_id)
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(sale)
}

pub async fn complete_owner_sale(
    pool: &PgPool,
    sale_id: Uuid,
    returned_items: Vec<ReturnedItemRequest>,
    owner_id: Uuid,
) -> Result<OwnerSale, AppError> {
    let mut tx = pool.begin().await?;
    
    // Procesar returned_items (igual que worker_trips)
    // ...
    
    // Calcular totales
    let (sold_quantity, total_amount) = calculate_owner_sales(&loaded_items, &returned_items);
    
    // Actualizar sale
    let sale = sqlx::query_as::<_, OwnerSale>(
        r#"
        UPDATE owner_sales 
        SET return_time = NOW(),
            sold_quantity = $1,
            total_amount = $2,
            auto_withdrawal = $2
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&sold_quantity)
    .bind(&total_amount)
    .bind(&sale_id)
    .fetch_one(&mut *tx)
    .await?;
    
    // ═══ CRÍTICO: Registrar en caja (2 eventos) ═══
    
    // Obtener balance actual
    let current_balance = sqlx::query_scalar::<_, Option<Decimal>>(
        "SELECT balance FROM cash_register ORDER BY created_at DESC LIMIT 1 FOR UPDATE"
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .unwrap_or(Decimal::ZERO);
    
    // Evento 1: Ingreso
    sqlx::query(
        r#"
        INSERT INTO cash_register 
        (type, amount, balance, description, related_doc_type, related_doc_id, created_by)
        VALUES ('owner_sale', $1, $2, 'Venta del dueño', 'owner_sales', $3, $4)
        "#
    )
    .bind(&total_amount)
    .bind(&(current_balance + total_amount))
    .bind(&sale.id)
    .bind(&owner_id)
    .execute(&mut *tx)
    .await?;
    
    // Evento 2: Retiro inmediato
    sqlx::query(
        r#"
        INSERT INTO cash_register 
        (type, amount, balance, description, related_doc_type, related_doc_id, created_by)
        VALUES ('owner_withdrawal', $1, $2, 'Retiro automático por venta del dueño', 'owner_sales', $3, $4)
        "#
    )
    .bind(&(-total_amount))  // Negativo
    .bind(&current_balance)  // Vuelve al balance original
    .bind(&sale.id)
    .bind(&owner_id)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(sale)
}
```

### Por Qué Esta Separación

**Razón 1: Reportes de ventas**
```rust
// "Ventas totales del mes" - El dueño DEBE aparecer
pub async fn get_total_sales(pool: &PgPool, month: u32, year: i32) -> Result<i32, AppError> {
    let worker_sales = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT SUM(sold_quantity) FROM worker_trips WHERE EXTRACT(MONTH FROM departure_time) = $1"
    ).bind(&month).fetch_one(pool).await?;
    
    let owner_sales = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT SUM(sold_quantity) FROM owner_sales WHERE EXTRACT(MONTH FROM departure_time) = $1"
    ).bind(&month).fetch_one(pool).await?;
    
    Ok((worker_sales.unwrap_or(0) + owner_sales.unwrap_or(0)) as i32)
}
```

**Razón 2: Transparencia financiera**
- Balance de caja NO refleja ventas del dueño (él se llevó el dinero)
- Pero ganancias totales SÍ incluyen ventas del dueño
- Auditoría muestra exactamente cuánto retiró el dueño

---

## Conclusión

Este modelo de datos está diseñado para:
- **Trazabilidad total:** Cada cambio registrado, auditable
- **Precisión financiera:** Precios históricos, event sourcing en caja
- **Granularidad:** Control por congelador/producto/sabor/proveedor
- **Flexibilidad:** Manejo de casos especiales (deformados, dueño vendedor)
- **Performance:** Índices estratégicos y denormalización cuando tiene sentido
- **Integridad:** Transacciones SQL para operaciones atómicas
- **Concurrencia:** FOR UPDATE locks para evitar race conditions

**Stack Técnico:**
- **Backend:** Rust con Axum (web framework)
- **Base de datos:** PostgreSQL
- **ORM/Query builder:** SQLx (compile-time checked queries)
- **Serialización:** Serde
- **Manejo de decimales:** rust_decimal

**CRÍTICO:** Antes de implementar, lee este documento COMPLETO y entiende el "por qué" de cada decisión, no solo el "cómo".
