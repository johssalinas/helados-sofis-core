# 🍦 Helados Sofis - Backend Core

Backend REST API para el sistema de gestión de "Helados Sofis" en San Gil, Colombia. Construido con Rust + Axum + PostgreSQL siguiendo arquitectura hexagonal/clean.

## 🚀 Características

- ✅ **Arquitectura Hexagonal/Clean**: Separación por módulos con capas domain/application/infrastructure
- ✅ **Seguridad**: Autenticación Google OAuth + JWT (clave de 1 año)
- ✅ **Base de datos**: PostgreSQL con SQLx + migraciones automáticas
- ✅ **API REST**: 13 módulos con ~60 endpoints
- ✅ **Transaccionalidad**: Operaciones complejas con control de inventario y eventos de caja
- ✅ **Control de concurrencia**: Locks (`FOR UPDATE`) en operaciones críticas
- ✅ **Event Sourcing**: Sistema de caja registradora con balance calculado

## 📋 Prerrequisitos

- **Rust** 1.92.0 o superior
- **PostgreSQL** 14+ 
- **Google OAuth Client** (para autenticación)

## ⚙️ Configuración

### 1. Base de datos

```bash
# Crear base de datos
createdb helados_sofis

# Las migraciones se aplican automáticamente al iniciar el servidor
```

### 2. Variables de entorno

Crea un archivo `.env` en la raíz del proyecto basado en `.env.example`:

```env
# Database
DATABASE_URL=postgresql://usuario:password@localhost:5432/helados_sofis

# JWT Secret (generar con: openssl rand -base64 32)
JWT_SECRET=tu_secreto_super_largo_y_aleatorio_aqui

# Google OAuth
GOOGLE_CLIENT_ID=tu_client_id_de_google_oauth.apps.googleusercontent.com

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

### 3. Compilar y ejecutar

```bash
# Desarrollo
cargo run

# Producción (optimizado)
cargo build --release
./target/release/helados-sofis-core
```

El servidor iniciará en `http://0.0.0.0:3000`

## 🏗️ Estructura del Proyecto

```
src/
├── main.rs                    # Servidor Axum con todos los routers
├── shared/                    # Código compartido
│   ├── config.rs             # AppConfig + variables de entorno
│   ├── db.rs                 # Pool de conexiones PostgreSQL
│   ├── errors.rs             # AppError enum + IntoResponse
│   └── auth.rs               # JWT, AuthUser extractor, Role enum
└── modules/                   # 13 módulos de negocio
    ├── users/                # Gestión de usuarios internos
    ├── auth/                 # Login con Google OAuth
    ├── audit_log/            # Auditoría de acciones
    ├── catalog/              # Productos, proveedores, trabajadores, rutas
    ├── pricing/              # Precios y histórico
    ├── inventory/            # Control de stock por congelador
    ├── purchases/            # Compras a proveedores
    ├── worker_trips/         # Salidas y cierres de trabajadores
    ├── worker_payments/      # Pagos a trabajadores (reduce deuda)
    ├── cash_register/        # Caja registradora (event sourcing)
    ├── local_sales/          # Ventas en local
    ├── owner_sales/          # Ventas del propietario
    └── freezer_transfers/    # Transferencias entre congeladores
```

Cada módulo sigue la estructura:
```
module/
├── domain/
│   ├── entities.rs           # Entidades de negocio + DTOs
│   └── repositories.rs       # Traits de repositorios
├── application/
│   └── *.rs                  # Casos de uso
└── infrastructure/
    ├── persistence/
    │   └── postgres_repo.rs  # Implementación de repositorios
    └── controllers/
        └── http_router.rs    # Endpoints REST
```

## 📡 API Endpoints

Todos los endpoints están bajo el prefijo `/api`

### 🔐 Autenticación

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/auth/google` | Login con Google OAuth token | No |

**Body ejemplo:**
```json
{
  "token": "ya29.a0AfH6SMB..."
}
```

**Respuesta:**
```json
{
  "user": { "id": "uuid", "email": "...", "role": "admin" },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 👥 Usuarios (Users)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/users` | Listar todos los usuarios | Owner |
| POST | `/api/users` | Crear usuario | Owner |
| GET | `/api/users/:id` | Obtener usuario por ID | Owner |
| PATCH | `/api/users/:id` | Actualizar usuario | Owner |
| DELETE | `/api/users/:id` | Desactivar usuario | Owner |

### 📦 Catálogos (Catalog)

#### Productos
| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/products` | Listar productos activos | Owner/Admin |
| POST | `/api/products` | Crear producto | Owner |
| GET | `/api/products/:id` | Ver producto | Owner/Admin |
| PATCH | `/api/products/:id` | Actualizar producto | Owner |
| DELETE | `/api/products/:id` | Desactivar producto | Owner |

#### Proveedores
| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/providers` | Listar proveedores activos | Owner/Admin |
| POST | `/api/providers` | Crear proveedor | Owner |
| PATCH | `/api/providers/:id` | Actualizar proveedor | Owner |

#### Trabajadores
| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/workers` | Listar trabajadores activos | Owner/Admin |
| POST | `/api/workers` | Crear trabajador | Owner |
| GET | `/api/workers/:id` | Ver trabajador + deuda actual | Owner/Admin |
| PATCH | `/api/workers/:id` | Actualizar trabajador | Owner |

#### Rutas
| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/routes` | Listar rutas | Owner/Admin |
| POST | `/api/routes` | Crear ruta | Owner |
| PATCH | `/api/routes/:id` | Actualizar ruta | Owner |

#### Congeladores
| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/freezers` | Listar congeladores | Owner/Admin |
| POST | `/api/freezers` | Crear congelador | Owner |
| PATCH | `/api/freezers/:id` | Actualizar congelador | Owner |

### 💰 Precios (Pricing)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/pricing/current` | Obtener precios actuales | Owner/Admin |
| POST | `/api/pricing` | Establecer nuevo precio (crea historial) | Owner |
| GET | `/api/pricing/history/:product_id` | Ver historial de precios de un producto | Owner |

### 📊 Inventario (Inventory)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/inventory` | Ver todo el inventario | Owner/Admin |
| GET | `/api/inventory/:id` | Ver item específico | Owner/Admin |
| GET | `/api/inventory/by-freezer/:freezer_id` | Inventario de un congelador | Owner/Admin |

### 🛒 Compras (Purchases)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/purchases` | Listar compras con filtros opcionales | Owner |
| POST | `/api/purchases` | Registrar compra (suma inventario) | Owner |
| GET | `/api/purchases/:id` | Ver compra con items | Owner |

**Body ejemplo POST:**
```json
{
  "provider_id": "uuid",
  "total_amount": "1500000.00",
  "invoice_number": "FAC-001",
  "notes": "Compra quincenal",
  "items": [
    {
      "product_id": "uuid",
      "freezer_id": "uuid",
      "quantity": 100,
      "unit_cost": "1500.00"
    }
  ]
}
```

### 🚚 Salidas de Trabajadores (Worker Trips)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/worker-trips` | Crear salida (resta inventario) | Owner/Admin |
| GET | `/api/worker-trips/active` | Salidas abiertas | Owner/Admin |
| GET | `/api/worker-trips/:id` | Ver salida con items | Owner/Admin |
| POST | `/api/worker-trips/:id/complete` | Cerrar salida (calcula ventas, devueltos, deformados, actualiza deuda) | Owner/Admin |
| GET | `/api/worker-trips/by-worker/:worker_id` | Historial de salidas de un trabajador | Owner/Admin |
| GET | `/api/worker-trips/by-date-range` | Buscar por rango de fechas | Owner/Admin |

**Complete body ejemplo:**
```json
{
  "returned_items": [
    { "product_id": "uuid", "quantity": 5, "deformed": 2 }
  ],
  "cash_collected": "50000.00",
  "notes": "Ruta completada sin novedades"
}
```

### 💸 Pagos a Trabajadores (Worker Payments)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/worker-payments` | Registrar pago (reduce deuda, evento en caja) | Owner |
| GET | `/api/worker-payments/:worker_id` | Historial de pagos de un trabajador | Owner |

### 🏪 Ventas Locales (Local Sales)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/local-sales` | Registrar venta local (resta inventario, evento en caja) | Owner/Admin |
| GET | `/api/local-sales` | Listar ventas locales | Owner/Admin |
| GET | `/api/local-sales/:id` | Ver venta con items | Owner/Admin |

**Body ejemplo:**
```json
{
  "sale_type": "efectivo",  // "efectivo", "transferencia", "regalo"
  "items": [
    { "product_id": "uuid", "freezer_id": "uuid", "quantity": 10 }
  ]
}
```

### 👔 Ventas del Propietario (Owner Sales)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/owner-sales` | Crear salida del propietario | Owner |
| GET | `/api/owner-sales/active` | Salidas abiertas del owner | Owner |
| POST | `/api/owner-sales/:id/complete` | Cerrar salida (calcula ventas, 2 eventos en caja: ingreso + retiro automático) | Owner |
| GET | `/api/owner-sales/:id` | Ver salida con items | Owner |

### 🔄 Transferencias entre Congeladores (Freezer Transfers)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| POST | `/api/freezer-transfers` | Transferir productos entre congeladores | Owner/Admin |
| GET | `/api/freezer-transfers` | Listar transferencias | Owner |
| GET | `/api/freezer-transfers/:id` | Ver transferencia con items | Owner |

### 💵 Caja Registradora (Cash Register)

| Método | Ruta | Descripción | Auth |
|--------|------|-------------|------|
| GET | `/api/cash-register/balance` | Balance actual de caja | Owner/Admin |
| GET | `/api/cash-register/today` | Transacciones del día | Owner/Admin |
| GET | `/api/cash-register/range` | Transacciones en rango de fechas | Owner/Admin |
| POST | `/api/cash-register/expense` | Registrar gasto | Owner |
| POST | `/api/cash-register/withdrawal` | Retiro de efectivo | Owner |

**Tipos de transacciones (event sourcing):**
- `ingreso`: Ingresos de ventas locales/trabajador
- `retiro`: Retiros por dueño o automáticos (owner sales)
- `gasto`: Gastos operativos
- `pago_trabajador`: Pagos a trabajadores
- `ajuste`: Ajustes manuales

## 🔒 Sistema de Permisos

### Roles
- **Owner**: Acceso total (endpoints con `require_owner()`)
- **Admin**: Acceso operativo (endpoints con `require_role(Admin)`)

### Headers de autenticación
```
Authorization: Bearer <JWT_TOKEN>
```

## 🗄️ Base de Datos

### Esquema principal (24 tablas)

- `users`: Usuarios internos del sistema
- `products`, `providers`, `workers`, `routes`, `freezers`: Catálogos
- `price_history`: Histórico de precios (temporal data pattern)
- `inventory`: Stock actual por producto + congelador
- `purchases` + `purchase_items`: Compras a proveedores
- `worker_trips` + `worker_trip_items`, `returned_items`: Salidas de trabajadores
- `worker_payments`: Pagos a trabajadores
- `cash_transactions`: Event sourcing de caja registradora
- `local_sales` + `local_sale_items`: Ventas locales
- `owner_sales` + `owner_sale_loaded_items`, `owner_sale_returned_items`: Ventas del propietario
- `freezer_transfers` + `transfer_items`: Transferencias entre congeladores
- `audit_log`: Auditoría de acciones

### Migraciones automáticas
Las migraciones se aplican automáticamente al iniciar el servidor usando SQLx:
```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

## 🧪 Testing

```bash
# Compilar en modo check (rápido)
cargo check

# Compilar con warnings
cargo build

# Compilar para producción
cargo build --release

# Formatear código
cargo fmt

# Linter
cargo clippy
```

## 📦 Dependencias Principales

- **axum**: Web framework
- **tokio**: Runtime async
- **sqlx**: Base de datos con compile-time checked queries
- **jsonwebtoken**: JWT encoding/decoding
- **reqwest**: HTTP client para Google OAuth
- **uuid**: Generación de UUIDs v4
- **chrono**: Manejo de fechas
- **rust_decimal**: Aritmética decimal precisa para dinero
- **tower-http**: CORS + tracing
- **serde**: Serialización/deserialización JSON

## 🚀 Despliegue

### Docker (próximamente)

```dockerfile
FROM rust:1.92 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/helados-sofis-core /usr/local/bin/
CMD ["helados-sofis-core"]
```

### Variables de entorno en producción

```env
DATABASE_URL=postgresql://user:pass@db:5432/helados_sofis
JWT_SECRET=<generar con openssl rand -base64 64>
GOOGLE_CLIENT_ID=<tu_client_id>
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

## 📝 Notas de Implementación

### Patrones de Diseño
- **Hexagonal/Clean Architecture**: Separación domain/application/infrastructure
- **Repository Pattern**: Abstracción de persistencia con traits
- **Event Sourcing**: Sistema de caja registradora como secuencia de eventos
- **Temporal Data**: price_history mantiene histórico con fechas válidas
- **UPSERT**: Inventario usa INSERT ... ON CONFLICT UPDATE
- **Transaccionalidad**: Operaciones complejas con SQLx transactions
- **Denormalización controlada**: worker.current_debt, worker.total_sales

### Lógica de Negocio Clave
1. **Worker Trips**: Salida resta inventario → Cierre calcula ventas (cargados - devueltos), suma deuda trabajador, registra ingreso en caja
2. **Owner Sales**: Salida resta inventario → Cierre registra 2 eventos: ingreso de ventas + retiro automático (balance vuelve al original)
3. **Cash Register**: Balance denormalizado vs calculated con FOR UPDATE para prevenir race conditions
4. **Inventory**: UPSERT con lookup de provider_id en transferencias

## 👨‍💻 Autor

Desarrollado por **Johs Salinas**

---
