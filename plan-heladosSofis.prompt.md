# Plan: Sistema de Gestión de Inventario y Ventas para Heladería "Helados Sofis"

## Contexto del Negocio

**Helados Sofis** es un negocio familiar de venta de helados ubicado en San Gil, Colombia. El dueño necesita digitalizar y optimizar la gestión de su operación que actualmente se maneja manualmente en un libro de registro.

### Modelo de Negocio Actual

**Ubicación y Estructura:**

- Local principal en San Gil con 7 congeladores de capacidades variables
- Negocio familiar donde el dueño también trabaja como vendedor
- Entre semana: ~3 trabajadores activos
- Fines de semana: ~6 trabajadores activos (varía por días)

**Productos:**

- 3 tipos principales: Choco Conos, Vasos de helado, Paletas
- Cada tipo tiene múltiples sabores (dinámicos, cambian con cada pedido del proveedor)
- Los sabores son específicos por proveedor (importante para trazabilidad)
- Productos se identifican por: Tipo + Sabor + Proveedor

**Operación de Proveedores:**

- Se realizan pedidos aproximadamente 1 vez por semana
- El pedido se hace cuando solo quedan 1-2 congeladores llenos (para dar tiempo a congelación)
- Algunos proveedores entregan en el local, otros requieren ir a recoger
- Pago generalmente al momento, pero ocasionalmente a crédito
- Los sabores se registran al momento de la compra (no están predefinidos en el sistema)
- Puede haber entregas de varios proveedores el mismo día

**Flujo de Trabajo de Vendedores:**

1. **Carga de Helados:**

   - Los trabajadores cargan helados en su propia "caba" (nevera portátil personal)
   - Capacidad típica: ~100 helados, pero varía según la caba del trabajador
   - El trabajador mismo elige qué tipos y sabores llevar
   - Pueden cargar de varios congeladores en una misma salida
   - El administrador saca físicamente los helados y LUEGO registra en el sistema
   - Hora de salida: OBLIGATORIA
   - Ruta/pueblo destino: OPCIONAL (pero importante para análisis)
   - Pueden hacer varias salidas en el mismo día (se registra cada una por separado)

2. **Venta en Ruta:**

   - Cada trabajador tiene rutas fijas (pueblos y fincas cercanos a San Gil)
   - Se busca evitar que 2 trabajadores vayan a la misma ruta (afecta ventas)
   - Precios fijos definidos por el negocio (no negociables por trabajador)
   - El trabajador crea/selecciona rutas (se almacenan para autocomplete futuro)

3. **Devolución:**

   - Hora de regreso: OPCIONAL
   - El administrador cuenta físicamente los helados devueltos
   - Los helados pueden estar:
     - **Buenos:** Vuelven al inventario normal
     - **Deformados:** Se derritieron parcialmente y se deformaron al recongelar
   - Helados deformados:
     - Se guardan en canasta física con papel del nombre del trabajador
     - Se asignan en el sistema al trabajador responsable
     - Ese trabajador debe venderlos en su próxima salida (puede llevar parciales, no todos)
     - Si el trabajador no vuelve/renuncia: se mezclan con inventario o se venden local/regalados
   - Los devueltos pueden ir a CUALQUIER congelador (no necesariamente de donde salieron)
   - Sistema calcula automáticamente: `Deuda = (Cargados - Devueltos) × Precio_Base`

4. **Pago:**
   - Fórmula: El trabajador paga `(Cantidad_Vendida × Precio_Base)` donde Precio_Base es la parte que se queda el negocio
   - La comisión del trabajador es la diferencia entre Precio_Ruta y Precio_Base
   - Ejemplo: Precio_Ruta = $2000, Precio_Base = $1400 → Trabajador paga $1400, se queda con $600 de comisión
   - Momento de pago:
     - Si devolvió helados: paga INMEDIATAMENTE en efectivo
     - Si vendió todo: puede pagar la próxima vez que venga a cargar (sin plazo fijo)
   - Pagos parciales: SÍ permitidos
   - Registro: fecha, monto, saldo anterior, saldo nuevo
   - La deuda se acumula sin límite (no hay bloqueo por deuda)

**Caso Especial: Dueño como Vendedor:**

- El dueño también sale a vender a rutas
- Se registra igual: carga, devolución, productos vendidos
- DIFERENCIA CRÍTICA: NO genera deuda, NO paga
- El dinero de sus ventas va directo a "ganancia del negocio"
- Flujo financiero:
  1. Se registra venta con monto total
  2. Ingresa a caja
  3. INMEDIATAMENTE se registra retiro del dueño por el mismo monto
  4. Resultado neto en caja = 0, pero queda registrado para reportes de ventas

**Ventas en Local:**

- Muy esporádicas (pocas personas compran en el local)
- Precio local: MÁS BARATO que precio de ruta (se ahorra la comisión del trabajador)
- Precio personalizado: PERMITIDO por administrador con campo de observaciones
- Atiende: el administrador
- Se descuenta del congelador que el administrador elija
- Tipos:
  - Venta local normal (precio local)
  - Venta precio personalizado (con observaciones)
  - Regalo/consumo familiar (precio = 0, pero se registra para inventario)

**Gestión de Congeladores:**

- 7 congeladores numerados (1, 2, 3, 4, 5, 6, 7), pueden ser más o menos en el futuro
- Capacidad máxima VARÍA por congelador (hay grandes y pequeños)
- Organización: generalmente por tipo de producto, pero a veces mezclados
- **Optimización de electricidad:**
  - Cuando ≥2 congeladores están semi-vacíos y sus contenidos caben en 1 solo congelador:
    - Sistema alerta para transferir productos y apagar congeladores vacíos
  - Cálculo: Suma de contenidos de congeladores semi-vacíos ≤ Capacidad máxima de 1 congelador
  - Ejemplo: Congelador 1 (20 conos, 10 vasos) + Congelador 2 (30 conos, 15 vasos) = 50 conos, 25 vasos
    - Si Congelador 3 tiene capacidad de 100 conos y 50 vasos → ALERTA para consolidar
- Registro de encendido/apagado con timestamp
- Transferencias entre congeladores: registra origen, destino, productos, cantidades, razón

**Sistema de Precios:**
Cada producto (Tipo + Sabor + Proveedor) tiene 4 precios:

1. **Costo Proveedor:** Lo que le cuesta al negocio
2. **Precio Base:** Lo que el trabajador paga al negocio (Negocio gana comisión)
3. **Precio Ruta:** Lo que el trabajador cobra al cliente final (Trabajador gana comisión)
4. **Precio Local:** Venta en local (más barato, sin comisión trabajador)

- Los proveedores pueden cambiar precios → Se actualiza hacia adelante
- **Historial de precios:** Se guarda con timestamp `effectiveDate`
- Reportes deben usar el precio histórico según fecha de venta
- La comisión del trabajador VARÍA por producto (no es porcentaje fijo)

**Control Financiero:**

- **Caja registradora:**
  - Ingresos: pagos de trabajadores, ventas locales, ventas del dueño (inmediatamente retiradas)
  - Egresos: retiros del dueño, gastos operativos
- **Gastos operativos (solo dueño puede registrar):**
  - Categorizados: luz, agua, mantenimiento congeladores, transporte, otros
  - Con descripción y monto
- **Retiros del dueño:**
  - Para gastos personales
  - Con campo de observaciones
  - Afectan "dinero disponible en caja"
- **Utilidad neta:** Ingresos - Gastos operativos - Retiros

**Cuentas por Pagar a Proveedores:**

- Si compran a crédito: se registra con monto pendiente
- NO necesitan fecha límite de pago
- NO necesitan alertas de vencimiento
- Control simple: monto adeudado, fecha de pago cuando se liquida

### Problemas Actuales (Sistema Manual con Libro)

1. **Ineficiencia:** El administrador debe escribir todo a mano
2. **Errores de cálculo:** Al sumar ventas, calcular deudas, contar inventario
3. **Sin visibilidad en tiempo real:** El dueño no sabe estado del negocio desde su casa
4. **Imposible generar reportes:** No pueden analizar tendencias, mejores trabajadores, productos más vendidos
5. **Riesgo de pérdida de información:** Si se pierde el libro
6. **No hay control de congeladores:** No saben cuándo es óptimo apagar uno para ahorrar luz
7. **Difícil rastrear helados deformados:** Se pierden o no se asignan correctamente al trabajador responsable
8. **Sin alertas:** No saben cuándo se está acabando un producto hasta que ya es tarde

### Objetivos del Sistema

1. **Digitalizar operación completa:** Desde compra a proveedor hasta venta final
2. **Calcular automáticamente:** Deudas, comisiones, niveles de inventario, consolidación de congeladores
3. **Visibilidad remota para dueño:** Ver estado del negocio desde cualquier lugar
4. **Reportes y análisis:** Ventas por periodo, mejores trabajadores, productos más vendidos, rentabilidad por ruta
5. **Alertas inteligentes:** Stock bajo, congeladores consolidables, trabajadores con deformados pendientes
6. **Trazabilidad completa:** Auditoría de quién hizo qué y cuándo
7. **Cero costo operativo:** Servicios gratuitos (rust free tier)

## Resumen Ejecutivo

Sistema móvil multiplataforma (Flutter) para administrar inventario en múltiples congeladores, gestionar trabajadores vendedores, controlar deudas, calcular comisiones automáticamente, rastrear ventas por rutas, y generar reportes con gráficos. Backend personalizado en Rust con autenticación Google (única), control de acceso basado en roles (RBAC) validado en el backend, base de datos PostgreSQL, dos roles (Dueño/Admin) con permisos granulares, gestión de usuarios controlada por el Dueño, y auditoría completa de todas las operaciones.

## Steps

1. **Configurar infraestructura base**:

   - Crear proyecto Flutter con estructura de carpetas (`models/`, `services/`, `providers/`, `screens/`, `widgets/`)
   - Integrar con el backend de Rust a través de una API REST.
   - Configurar Google Sign-In en el cliente Flutter. El token de Google será enviado al backend de Rust para su validación y la creación de una sesión de usuario (JWT).
   - Implementar la lógica de validación de roles (Owner/Admin) en el backend de Rust, que se adjuntará al token de sesión.
   - Implementar un endpoint en la API de Rust para validar el email autorizado al crear una cuenta.
   - Implementar persistencia de datos en el cliente (offline-first) usando una base de datos local como Drift (sobre sqflite).
   - Definir el esquema de la base de datos local y la API REST para la colección `users` con campos: email, role, active, createdBy.
   - Establecer los índices necesarios en la base de datos PostgreSQL del backend.

2. **Implementar módulo de autenticación y control de acceso RBAC**:

   - Login con Google Sign-In (único método)
   - Validación de email autorizado post-login (el cliente envía el token de Google, el backend de Rust lo valida y consulta la base de datos de `users`).
   - Pantalla "Sin permisos" para emails no autorizados, mostrada por el cliente si el backend devuelve un error de autorización.
   - Gestión de usuarios (solo Owner: a través de endpoints seguros en la API de Rust).
   - Sistema de roles (owner/admin) con permisos diferenciados, aplicados en el backend de Rust.
   - Guards de navegación en Flutter (route protection) basados en el rol obtenido del backend.
   - Renderizado condicional de widgets según rol.
   - Lógica de autorización en cada endpoint de la API de Rust para validar el rol y la operación.
   - Endpoint de API para validar el acceso al crear la cuenta.
   - Re-validación de sesión (token JWT) al iniciar la app.
   - Pantalla de perfil con información de rol.
   - Sistema de auditoría automática en el backend que registra quién hizo cada cambio con timestamps.

3. **Desarrollar gestión de catálogo base**: CRUD de Tipos de Productos dinámicos (Paletas/Conos/Vasos), Sabores por tipo, Proveedores, Trabajadores (con datos de contacto, estado activo/inactivo), Rutas/Pueblos autocomplete, y Congeladores (número, capacidad máxima variable). Incluir historial de precios por producto con timestamps

4. **Construir sistema de inventario multi-congelador**: Registro de compras a proveedores (tipo, sabor, cantidad, precio, congelador destino, pago inmediato/crédito), visualización de stock por congelador con nivel de ocupación (%), transferencias entre congeladores (con registro de qué/cuánto/origen/destino), y alertas automáticas cuando suma de congeladores semi-vacíos quepa en uno solo para apagar y ahorrar electricidad

5. **Implementar flujo completo de trabajadores**: Registro de carga de helados (trabajador selecciona productos/sabores/cantidades de múltiples congeladores, hora de salida obligatoria, ruta opcional, alerta si tiene helados deformados pendientes), registro de devolución (admin cuenta devueltos por tipo/sabor, marca deformados asignándolos al trabajador responsable, elige congelador destino), cálculo automático de deuda `(cargados - devueltos) × precio_base`, y sistema de pagos parciales con historial y saldo actual visible

6. **Desarrollar módulo financiero y reportes**: Ventas en local (admin elige productos/sabores/congelador, precio local/personalizado con observaciones, consumo familiar/regalos), control de caja (dinero disponible, retiros del dueño con observaciones, gastos operativos categorizados solo por dueño), dashboard con gráficos (dinero en caja, ventas último mes, top trabajadores por cantidad vendida), y reportes personalizables (diario/semanal/mensual/custom, por producto, por trabajador, rentabilidad por ruta, comparación entre periodos, exportable visualmente)

7. **Implementar sistema de notificaciones y alertas**: Alertas in-app para dueño (stock bajo configurable por producto, congeladores consolidables, trabajadores con helados deformados), notificaciones de encendido/apagado de congeladores, y recordatorios de pedidos a proveedores cuando quedan 1-2 congeladores llenos

8. **Testing, optimización y despliegue**: Tests unitarios de cálculos críticos (pagos, consolidación congeladores), tests de widgets clave, configurar índices en la base de datos PostgreSQL para performance, implementar paginación en las APIs y en las listas del cliente, generar APK para Android, configurar un sistema de distribución de betas (ej. GitHub Releases), y documentar la configuración y despliegue del backend de Rust.

## Decisiones Técnicas Fundamentales y Razonamiento

### 1. Granularidad del Inventario: Por (Congelador, Tipo, Sabor, Proveedor)

**Decisión:** El inventario se rastrea a nivel INDIVIDUAL por cada combinación única de:

- Congelador (1-7)
- Tipo de Producto (Paleta/Cono/Vaso)
- Sabor (Fresa, Chocolate, etc.)
- Proveedor (porque puede haber "Paleta de Fresa" de diferentes proveedores con precios distintos)

**Razonamiento:**

- **Precisión total:** Permite saber EXACTAMENTE cuántas "Paletas de Fresa del Proveedor X" hay en el "Congelador 3"
- **Alertas granulares:** Alertar cuando se acaba un sabor específico, no solo un tipo genérico
- **Reportes precisos:** Saber qué sabores se venden más, qué proveedor tiene mejor rotación
- **Trazabilidad de deformados:** Asignar helados deformados al trabajador requiere saber sabor exacto
- **Precios por proveedor:** Cada combinación puede tener precio diferente (crítico para cálculos)
- **Consolidación de congeladores:** Calcular si caben productos requiere saber cantidades exactas por tipo

**Alternativa rechazada:** Inventario solo por "Tipo" (sin sabor/proveedor)

- **Por qué se rechazó:** Imposible rastrear deformados por trabajador, no se puede alertar por sabor específico, precios serían incorrectos

### 2. Flujo Especial: Dueño como Vendedor

**Problema:** El dueño también vende helados pero se queda con TODO el dinero (no paga comisión al negocio porque ÉL ES el negocio).

**Decisión de implementación:**

1. **Registrar operación completa:**

   - Carga de helados (igual que trabajador normal)
   - Devolución de helados (igual que trabajador normal)
   - Cálculo de productos vendidos (igual)

2. **NO generar deuda:**

   - El campo `workerId` en `ownerSales` referencia al dueño
   - NO se crea registro en `workerPayments`
   - NO se suma a deuda del dueño

3. **Manejo financiero especial:**

   - Se registra venta con monto total en `ownerSales`
   - Se crea entrada en `cashRegister`:
     ```
     Tipo: owner_sale
     Monto: +$50,000 (ingreso)
     ```
   - INMEDIATAMENTE se crea segunda entrada:
     ```
     Tipo: owner_withdrawal
     Monto: -$50,000 (retiro automático)
     Descripción: "Retiro automático por venta del dueño"
     ```
   - **Resultado neto en caja:** $0
   - **PERO:** Queda registrado para reportes de ventas totales

4. **Visibilidad en reportes:**
   - Aparece como "Venta del Dueño" en reportes de ventas
   - Suma a "Ventas Totales" del negocio
   - Suma a "Ganancias" (porque es ganancia real)
   - NO afecta "Saldo de Caja Disponible" (porque se retiró inmediatamente)

**Razonamiento:**

- El dueño necesita saber cuánto vendió ÉL específicamente (para compararse con trabajadores)
- El inventario DEBE actualizarse correctamente (los helados sí se vendieron)
- Las estadísticas de "mejor vendedor" deben incluir al dueño
- La caja NO debe mostrar dinero que nunca estuvo ahí
- Transparencia total: queda registro de que el dueño tomó ese dinero

### 3. Sistema RBAC (Role-Based Access Control)

**Decisión:** 2 roles únicos con permisos MUY diferentes, validados tanto en frontend (UI) como en el backend (Rust).

#### Matriz de Control de Acceso (RBAC)

| Módulo                   | Vista/Funcionalidad           | Dueño (Owner) | Admin | Descripción                              |
| ------------------------ | ----------------------------- | ------------- | ----- | ---------------------------------------- |
| **AUTH**                 | Iniciar sesión con Google     | ✅            | ✅    | Todos los usuarios autorizados           |
|                          | Ver perfil propio             | ✅            | ✅    | Información personal                     |
|                          | Gestionar usuarios            | ✅            | ❌    | Agregar/editar/eliminar usuarios y roles |
| **DASHBOARD**            | Ver dashboard principal       | ✅            | ❌    | Métricas, gráficos, KPIs                 |
|                          | Ver dinero en caja            | ✅            | ❌    | Balance actual de caja                   |
|                          | Ver alertas                   | ✅            | ❌    | Notificaciones del sistema               |
| **CATÁLOGO**             | Ver productos/sabores         | ✅            | ✅    | Lista de catálogo                        |
|                          | Crear/editar productos        | ✅            | ❌    | CRUD de tipos de productos               |
|                          | Crear/editar sabores          | ✅            | ❌    | CRUD de sabores                          |
|                          | Crear/editar proveedores      | ✅            | ❌    | CRUD de proveedores                      |
|                          | Gestionar precios             | ✅            | ❌    | Configurar precios de productos          |
| **TRABAJADORES**         | Ver lista de trabajadores     | ✅            | ✅    | Solo datos básicos                       |
|                          | Ver deuda de trabajadores     | ✅            | ✅    | Saldo actual                             |
|                          | Crear/editar trabajadores     | ✅            | ❌    | Datos de contacto, estado                |
|                          | Ver historial completo        | ✅            | ❌    | Viajes, pagos, estadísticas              |
| **RUTAS**                | Ver rutas existentes          | ✅            | ✅    | Lista de pueblos/zonas                   |
|                          | Crear rutas                   | ✅            | ✅    | Al registrar salidas                     |
| **CONGELADORES**         | Ver estado de congeladores    | ✅            | ✅    | Nivel de ocupación                       |
|                          | Crear/editar congeladores     | ✅            | ❌    | Configurar capacidades                   |
|                          | Ver alertas consolidación     | ✅            | ❌    | Sugerencias de ahorro                    |
|                          | Registrar encendido/apagado   | ✅            | ✅    | Control eléctrico                        |
| **INVENTARIO**           | Ver inventario actual         | ✅            | ✅    | Stock por congelador                     |
|                          | Registrar compras             | ✅            | ✅    | Ingresos de proveedor                    |
|                          | Transferir entre congeladores | ✅            | ✅    | Mover productos                          |
|                          | Ver historial de compras      | ✅            | ❌    | Reportes históricos                      |
|                          | Editar/eliminar registros     | ✅            | ❌    | Corrección de errores                    |
| **SALIDAS TRABAJADORES** | Registrar carga de helados    | ✅            | ✅    | Salida de trabajador                     |
|                          | Registrar devolución          | ✅            | ✅    | Retorno de trabajador                    |
|                          | Ver viajes activos            | ✅            | ✅    | Salidas sin retorno                      |
|                          | Ver historial de viajes       | ✅            | ❌    | Reportes completos                       |
|                          | Editar/eliminar viajes        | ✅            | ❌    | Corrección de errores                    |
| **PAGOS**                | Registrar pago de trabajador  | ✅            | ✅    | Abonar a deuda                           |
|                          | Ver historial de pagos        | ✅            | ❌    | Todos los pagos                          |
|                          | Editar/eliminar pagos         | ✅            | ❌    | Corrección de errores                    |
| **VENTAS LOCALES**       | Registrar venta local         | ✅            | ✅    | Venta en el local                        |
|                          | Precio personalizado          | ✅            | ✅    | Con observaciones                        |
|                          | Ver historial ventas          | ✅            | ❌    | Reportes completos                       |
| **VENTAS DUEÑO**         | Registrar salida del dueño    | ✅            | ❌    | Dueño vendiendo                          |
|                          | Ver historial dueño           | ✅            | ❌    | Solo el dueño                            |
| **CAJA**                 | Ver balance de caja           | ✅            | ❌    | Dinero disponible                        |
|                          | Ver movimientos de caja       | ✅            | ❌    | Historial completo                       |
|                          | Registrar gastos operativos   | ✅            | ❌    | Luz, agua, mantenimiento                 |
|                          | Registrar retiros             | ✅            | ❌    | Retiros personales                       |
|                          | Editar/eliminar movimientos   | ✅            | ❌    | Corrección de errores                    |
| **REPORTES**             | Ver reportes de ventas        | ✅            | ❌    | Por periodo, producto, trabajador        |
|                          | Ver reportes financieros      | ✅            | ❌    | Rentabilidad, ganancias                  |
|                          | Ver reportes de inventario    | ✅            | ❌    | Rotación, stock                          |
|                          | Comparar periodos             | ✅            | ❌    | Análisis temporal                        |
|                          | Exportar reportes             | ✅            | ❌    | Visuales o datos                         |
| **AUDITORÍA**            | Ver logs de auditoría         | ✅            | ❌    | Quién hizo qué y cuándo                  |
|                          | Filtrar por usuario           | ✅            | ❌    | Investigación                            |
| **NOTIFICACIONES**       | Ver notificaciones            | ✅            | ❌    | Alertas del sistema                      |
|                          | Marcar como leídas            | ✅            | ❌    | Gestión de alertas                       |

#### Rol: Dueño (Owner)

**Descripción:** Propietario del negocio con control total del sistema.

**Permisos de Frontend:**

- ✅ Acceso a TODAS las pantallas
- ✅ Ver TODOS los datos (inventario, ventas, reportes, deudas, caja, gastos)
- ✅ Realizar TODAS las operaciones (crear, editar, eliminar)
- ✅ Gestionar usuarios (agregar/eliminar/cambiar roles)
- ✅ Ver logs de auditoría completos
- ✅ Configurar catálogo (productos, sabores, proveedores, precios)
- ✅ Configurar congeladores
- ✅ Ver y gestionar reportes financieros
- ✅ Registrar gastos operativos y retiros
- ✅ Recibir notificaciones y alertas
- ✅ Corregir errores históricos (editar/eliminar registros antiguos)

**Permisos de Backend (Rust):**

- ✅ Read: Acceso a endpoints `GET` para TODAS las colecciones.
- ✅ Write: Acceso a endpoints `POST`/`PUT` para TODAS las colecciones.
- ✅ Delete: Acceso a endpoints `DELETE` para TODAS las colecciones.
- ✅ Bypass de validaciones de negocio (puede modificar cualquier timestamp a través de parámetros especiales en la API).

**Acceso:**

- Desde CUALQUIER dispositivo
- Desde CUALQUIER ubicación (casa, local, ruta)
- 24/7 sin restricciones

**Responsabilidades:**

- Tomar decisiones de negocio
- Supervisar operación completa
- Analizar reportes y métricas
- Gestionar finanzas
- Agregar/remover personal del sistema

#### Rol: Administrador (Admin)

**Descripción:** Personal de confianza que opera el local, sin acceso a información financiera sensible.

**Permisos de Frontend:**

- ✅ Ver inventario actual por congelador
- ✅ Ver lista de trabajadores (solo nombre, teléfono, deuda actual)
- ✅ Registrar compras a proveedores
- ✅ Registrar carga de helados a trabajadores
- ✅ Registrar devolución de trabajadores
- ✅ Registrar pagos de trabajadores
- ✅ Registrar ventas locales
- ✅ Transferir helados entre congeladores
- ✅ Registrar encendido/apagado de congeladores
- ❌ NO puede ver dashboard con gráficos
- ❌ NO puede ver reportes financieros
- ❌ NO puede ver dinero total en caja
- ❌ NO puede ver historial completo de trabajadores
- ❌ NO puede ver gastos operativos
- ❌ NO puede ver retiros del dueño
- ❌ NO puede ver ventas del dueño
- ❌ NO puede ver logs de auditoría
- ❌ NO puede configurar catálogo (productos, sabores, precios)
- ❌ NO puede gestionar usuarios
- ❌ NO puede editar/eliminar registros antiguos (solo crear nuevos)
- ❌ NO recibe notificaciones

**Permisos de Backend (Rust):**

- ✅ Read: Endpoints `GET` para: `inventory`, `workers`, `freezers`, `products`, `flavors`, `providers`, `routes`.
- ✅ Write (solo CREATE): Endpoints `POST` para: `purchases`, `workerTrips`, `workerPayments`, `localSales`, `freezerTransfers`.
- ❌ NO read: Sin acceso a endpoints para: `cashRegister`, `ownerSales`, `auditLogs`, `users`, `priceHistory`, reportes financieros.
- ❌ NO update/delete: Sin acceso a endpoints `PUT`/`DELETE` (no puede modificar registros existentes).

**Acceso:**

- Solo desde dispositivos del local
- Durante horario de operación
- Funciones operativas básicas

**Responsabilidades:**

- Atención en el local
- Registro de operaciones diarias
- Control de salidas/retornos de trabajadores
- Cobro a trabajadores

#### Trabajadores Vendedores: SIN acceso al sistema

**Decisión:** Los trabajadores NO tienen usuario en el sistema

**Razonamiento:**

- Trabajadores solo necesitan saber cuánto deben (se les dice verbalmente)
- Agregar app para trabajadores:
  - Aumenta complejidad 3X
  - Requiere capacitación de trabajadores (personas de campo, no tech-savvy)
  - Riesgo de errores si trabajador manipula datos
  - No agrega valor real al negocio
- **Suficiente:** El admin les dice "vendiste X, debes Y"

#### Implementación de RBAC en Frontend (Flutter)

**Estrategia:** Guards + Widget Conditional Rendering

```dart
// 1. Route Guards
class RoleGuard {
  static bool canAccessRoute(String routeName, UserRole role) {
    final ownerRoutes = ['/dashboard', '/reports', '/cash', '/settings', ...];
    final adminRoutes = ['/inventory', '/workers/load', '/sales/local', ...];

    if (role == UserRole.owner) return true; // Owner accede a todo
    if (role == UserRole.admin) return adminRoutes.contains(routeName);
    return false;
  }
}

// 2. Widget Permissions
class PermissionWidget extends StatelessWidget {
  final List<UserRole> allowedRoles;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final userRole = ref.watch(authProvider).user?.role;
    if (allowedRoles.contains(userRole)) return child;
    return SizedBox.shrink(); // Oculta completamente
  }
}

// Uso:
PermissionWidget(
  allowedRoles: [UserRole.owner],
  child: IconButton(
    icon: Icon(Icons.delete),
    onPressed: () => deleteItem(),
  ),
)
```

**Flujo de Navegación:**

1. **Usuario inicia sesión con Google**
2. **Sistema verifica email en Base de datos:**
   - Si existe: obtiene `role`
   - Si no existe: muestra pantalla "Sin permisos"
3. **Usuario con rol válido entra al sistema**
4. **Router verifica rol antes de cada navegación:**
   - Dueño: permite TODAS las rutas
   - Admin: solo rutas permitidas
   - Si intenta acceder a ruta no permitida: redirige a home o muestra mensaje
5. **UI renderiza condicionalmente:**
   - Botones de editar/eliminar solo para Owner
   - Menú lateral filtra opciones según rol
   - Tabs ocultas según permisos

#### Implementación de RBAC en Backend (Rust)

**Estrategia:** Middleware de autorización en cada endpoint de la API.

```rust
// Ejemplo conceptual de un middleware en un framework como Axum o Actix-web

// Middleware que extrae el token JWT, valida el rol y permite o deniega el acceso.
async fn role_guard(
    claims: Claims<TokenClaims>, // Extrae las reclamaciones del token JWT
    required_role: Role,
    next: Next,
) -> Result<Response, AppError> {
    if claims.role >= required_role {
        // Si el rol del usuario es suficiente
        Ok(next.run().await)
    } else {
        // Rol insuficiente, devuelve error de "Prohibido"
        Err(AppError::new(StatusCode::FORBIDDEN, "Permisos insuficientes"))
    }
}

// Aplicación del middleware a las rutas
let app = Router::new()
    // Rutas solo para Owner
    .route("/users", post(create_user).delete(delete_user))
    .route("/reports/financial", get(get_financial_report))
    .layer(middleware::from_fn_with_state(app_state.clone(), |claims, state| role_guard(claims, Role::Owner)))
    // Rutas para Admin y Owner
    .route("/inventory/purchase", post(register_purchase))
    .route("/workers/trip", post(create_worker_trip))
    .layer(middleware::from_fn_with_state(app_state.clone(), |claims, state| role_guard(claims, Role::Admin)));
```

````

#### Validaciones Adicionales de Seguridad

**1. Validación de Email Autorizado:**

```javascript
// Trigger: onCreate en Authentication
exports.validateUserAccess = functions.auth.user().onCreate(async (user) => {
  const email = user.email;

  // Buscar si el email está autorizado
  const userDoc = await admin
    .rust()
    .collection("users")
    .where("email", "==", email)
    .limit(1)
    .get();

  if (userDoc.empty) {
    // Email NO autorizado → Eliminar usuario de Auth
    await admin.auth().deleteUser(user.uid);
    throw new functions.https.HttpsError(
      "permission-denied",
      "Este email no tiene permisos para acceder al sistema."
    );
  }

  // Email autorizado → Actualizar lastLogin
  const userData = userDoc.docs[0].data();
  await admin
    .rust()
    .collection("users")
    .doc(userDoc.docs[0].id)
    .update({
      lastLogin: admin.rust.FieldValue.serverTimestamp(),
      displayName: user.displayName || userData.displayName,
      photoURL: user.photoURL || userData.photoURL,
    });
});
````

**2. Validación de Usuario Activo en Client:**

```dart
class AuthService {
  Future<UserModel?> validateUserAccess(User rustUser) async {
    final userDoc = await _rust
      .collection('users')
      .where('email', isEqualTo: rustUser.email)
      .limit(1)
      .get();

    if (userDoc.docs.isEmpty) {
      // Email no autorizado
      await _auth.signOut();
      throw UnauthorizedException('No tienes permisos para acceder');
    }

    final userData = UserModel.fromrust(userDoc.docs.first);

    if (!userData.active) {
      // Usuario desactivado
      await _auth.signOut();
      throw UnauthorizedException('Tu cuenta ha sido desactivada');
    }

    return userData;
  }
}
```

**3. Re-validación Periódica:**

```dart
// En cada inicio de app, verificar que el token JWT sigue siendo válido
class AppInitializer {
  Future<void> validateSession() async {
    final jwt = await _secureStorage.read(key: 'jwt');
    if (jwt == null) return;

    try {
      // Enviar el token al backend para validarlo y obtener el rol actualizado
      await _apiClient.get('/auth/validate_token');
    } catch (e) {
      // Si el token es inválido o expiró, cerrar sesión
      await _authService.signOut();
      navigatorKey.currentState?.pushReplacementNamed('/login');
    }
  }
}
```

#### Pantallas según Rol

**Pantallas de Owner:**

- Home / Dashboard (métricas, gráficos, alertas)
- Inventario (lista, compras, transferencias)
- Trabajadores (lista, historial, estadísticas, crear/editar)
- Ventas del Dueño (registrar salidas propias)
- Ventas Locales
- Caja (balance, movimientos, gastos, retiros)
- Reportes (ventas, financieros, inventario, comparaciones)
- Configuración (catálogo, precios, congeladores, usuarios)
- Auditoría (logs completos)
- Perfil

**Pantallas de Admin:**

- Hrust
  // En el endpoint de login/registro del backend en Rust
  async fn google_login(payload: GoogleTokenPayload) -> Result<AuthResponse, AppError> {
  // 1. Validar el token de Google con los servidores de Google
  let google_user_info = validate_google_token(payload.token).await?;

      // 2. Buscar si el email está en la base de datos de usuarios autorizados
      let user_record = db::find_user_by_email(&google_user_info.email).await?;

      match user_record {
          Some(user) if user.active => {
              // 3a. Usuario autorizado y activo: generar un token JWT de sesión
              let jwt = create_jwt_for_user(&user)?;
              // Actualizar lastLogin, etc.
              db::update_user_login_info(&user.id).await?;
              Ok(AuthResponse::new(jwt, user.role))
          }
          Some(_) => {
              // 3b. Usuario existe pero no está activo
              Err(AppError::new(StatusCode::FORBIDDEN, "Cuenta desactivada."))
          }
          None => {
              // 3c. Usuario no existe en la base de datos
              Err(AppError::new(StatusCode::FORBIDDEN, "Este email no tiene permisos."))
          }
      }

  }

````

**2. Validación de Usuario Activo en Client:**

```dart
class AuthService {
  Future<UserModel?> loginWithGoogle() async {
    // 1. Iniciar sesión con Google en el cliente para obtener el token
    final googleToken = await _googleSignIn.signIn();

    // 2. Enviar el token al backend de Rust
    final response = await _apiClient.post('/auth/google', data: {'token': googleToken});
el backend de Rust.
    if (response.statusCode == 200) {
      // 3. El backend validó y devolvió un token JWT y el rol del usuario
      final jwt = response.data['jwt'];
      final role = response.data['role'];

      // 4. Guardar el JWT de forma segura en el dispositivo
      await _secureStorage.write(key: 'jwt', value: jwt);

      // 5. Devolver el modelo de usuario para actualizar el estado de la app
      return UserModel(role: role, ...);
    } else {
      // El backend devolvió un error (permiso denegado, cuenta inactiva, etc.)
      await _googleSignIn.signOut();
      throw Exception(response.data['error']);
    }
  }
}
```        │         │         ▼
         │         │    Cerrar sesión
         │         │    Redirigir a Login
         │         │
         │         ▼
         │    ¿Usuario está activo?
         │         │         │
         │      SÍ │         │ NO
         │         │         ▼
         │         │    Mostrar "Cuenta desactivada"
         │         │    Cerrar sesión
         │         │
         │         ▼
         │    Cargar rol (owner/admin)
         │    Permitir acceso
         │         │
         ▼         ▼
    [Pantalla Login]  [Home según rol]
            │
            ▼
    [Botón Google Sign-In]
            │
    Cliente Flutter obtiene Token de Google
            │
            ▼
    Token se envía a API de Rust
            │
            ▼
    Backend valida Token con Google
         │         │
      SÍ │         │ NO
         │         ▼
         │    Devolver error
         │    "Token de Google inválido"
         │
         ▼
    Consultar DB (PostgreSQL):
    ¿Email existe en users?
         │         │
      SÍ │         │ NO
         │         ▼
         │    Devolver error
         │    "No tienes permisos"
         │
         ▼
    ¿Usuario está activo?
         │         │
      SÍ │         │ NO
         │         ▼
         │    Devolver error
         │    "Cuenta desactivada"
         │
         ▼
    Actualizar lastLogin en DB
    Generar y devolver Token JWT
            │
            ▼
    Cliente recibe JWT y Rol
    Cargar rol y permisos
            │
            ▼
    ¿Rol es owner?
         │         │
    SÍ   │         │ NO (admin)
         │         │
         ▼         ▼
    [Dashboard   [Home operativo
     completo]    simplificado]
````

**Explicación del flujo:** en la app Flutter.

- La app obtiene un token de autenticación de Google.

2. **Validación de permisos (en el backend de Rust):**

   - El cliente envía el token de Google a un endpoint seguro (`/auth/google`) en el backend de Rust.
   - El backend valida este token con los servidores de Google para confirmar su autenticidad.
   - Una vez validado, el backend busca el email del usuario en la tabla `users` de su base de datos PostgreSQL.
   - **Casos:**
     - **Email existe y usuario está activo:** El backend genera un token de sesión propio (JWT), lo devuelve al cliente junto con el rol del usuario.
     - **Email existe pero usuario inactivo:** El backend devuelve un error de "Cuenta desactivada".
     - **Email NO existe:** El backend devuelve un error de "Permiso denegado".
   - El cliente recibe la respuesta y actúa en consecuencia (guarda el JWT y navega a la pantalla principal, o muestra el error).

3. **Gestión de usuarios autorizados:**
   - SOLO el Dueño puede agregar/editar/eliminar usuarios a través de endpoints protegidos en la API.
   - Proceso de agregar usuario:
     1. Dueño va a Configuración → Usuarios → Agregar.
     2. La app llama al endpoint `POST /users`.
     3. El backend de Rust recibe el email y rol, y crea un nuevo registro en la tabla `users`.
     4. La persona agregada ahora puede iniciar sesión.
     5. Dueño ingresa email de la persona a agregar (ej: "maria@gmail.com")
     6. Dueño selecciona rol (owner/admin)
     7. Sistema crea documento en `users` con email + role + active=true
     8. María ahora puede iniciar sesión con su cuenta de Google "maria@gmail.com"

#### Estructura de Documento de Usuario

```rust
users/{userId}
  - email: string (usado para validación, ej: "admin@example.com")
  - displayName: string
  - photoURL: string
  - role: string ("owner" | "admin")
  - active: boolean (permite desactivar sin eliminar)
  - createdAt: timestamp
  - createdBy: userId (referencia a quién lo agregó)
  - lastLogin: timestamp
```

**Razonamiento del flujo:**

1. **Usuarios son personas de edad avanzada:**

   - Cita del cliente: "no van a recordar su contraseña"
   - Google Sign-In usa cuenta que ya tienen
   - 1 tap para entrar (vs recordar contraseña compleja)

2. **Seguridad mejorada:**

   - No hay contraseñas débiles ("123456")
   - No hay contraseñas compartidas
   - Google maneja 2FA si está habilitado
   - **Control total del dueño:** Solo emails autorizados pueden usar la app

3. **Recuperación trivial:**

   - Si olvidan cómo entrar: "botón de Google"
   - No hay flujo de "olvidé mi contraseña"

4. **Menos desarrollo:**

   - Email/password requiere:
     - Formulario de registro
     - Validación de email
     - Reset password flow
     - Manejo de contraseñas débiles
   - Google Sign-In: 3 líneas de código

5. **Persistencia de sesión:**

   - Sesión NO expira (usuario no tiene que iniciar sesión constantemente)
   - Justificación: "negocio pequeño, no hay muchos riesgos de seguridad"
   - Implementación: `persistence: LOCAL` en rust Auth
   - **Riesgo aceptado:** Si roban el celular, pueden acceder
   - **Mitigación:** Educación al usuario sobre bloqueo de pantalla del celular

6. **Escalabilidad de usuarios:**
   - Dueño puede agregar múltiples admins según crezca el negocio
   - Puede desactivar usuarios sin eliminar su historial
   - Puede cambiar roles si la persona cambia de responsabilidades

### 6. Auditoría: LOG de TODAS las operaciones CRUD

**Decisión:** Cada CREATE, UPDATE, DELETE se registra en colección `auditLogs`

**Qué se registra:**

- `userId`: Quién hizo el cambio
- `action`: "create", "update", "delete"
- `collection`: Qué colección se modificó
- `documentId`: Qué documento específico
- `changesBefore`: Estado anterior (solo en updates/deletes)
- `changesAfter`: Estado nuevo (solo en creates/updates)
- `timestamp`: Cuándo exactamente

**Razonamiento:**

1. **Negocio familiar = desconfianza potencial:**

   - Si hay problemas (ej: dinero faltante), el dueño puede investigar
   - "¿Quién registró esa venta? ¿A qué hora?"

2. **Corrección de errores:**

   - Si admin se equivoca, el dueño puede ver qué cambió
   - Puede revertir manualmente viendo el estado anterior

3. **Cumplimiento:**

   - Si en el futuro necesitan mostrar trazabilidad (impuestos, auditorías)

4. **Debugging:**
   - Si hay bug en la app, los logs ayudan a entender qué pasó

**Implementación:**

- Cloud Function trigger en cada colección importante
- O service en cliente que llama a `AuditService.log()` después de cada operación
- **Privacidad:** Solo el Dueño puede ver los logs (Security Rules)

### 7. Stock: Rastreo Granular con Estados Especiales

**Decisión:** Cada item de inventario tiene campos:

- `quantity`: Cantidad actual
- `isDeformed`: Boolean
- `assignedWorkerId`: Reference (solo si `isDeformed = true`)
- `minStockAlert`: Número configurable por producto

**Estados posibles:**

1. **Helado Normal:**

   - `isDeformed: false`
   - `assignedWorkerId: null`
   - Disponible para cualquier trabajador

2. **Helado Deformado Asignado:**

   - `isDeformed: true`
   - `assignedWorkerId: "worker123"`
   - SOLO ese trabajador puede/debe cargarlo
   - Sistema alerta cuando ese trabajador va a cargar

3. **Helado Deformado Liberado:**
   - Si trabajador renuncia/no vuelve
   - Admin puede:
     - Cambiar `assignedWorkerId: null` (liberar para cualquiera)
     - O eliminar (si se regala/vende local)

**Razonamiento:**

- Sistema físico ya existe (canastas con nombres)
- App debe reflejar realidad física
- Prevenir que trabajador cargue helados nuevos mientras tiene deformados pendientes
- Responsabilizar al trabajador que causó la deformación

### 8. Notificaciones: Solo In-App, Solo para Dueño

**Decisión:** NO push notifications, solo notificaciones dentro de la app

**Razonamiento:**

1. **Simplicidad:**

   - Push notifications requiere:
     - rust Cloud Messaging
     - Permisos de notificaciones en Android
     - Manejo de tokens
     - Testing en múltiples dispositivos
   - In-app: solo badge y lista

2. **Usuario objetivo:**

   - Solo el dueño recibe notificaciones
   - El dueño abre la app regularmente
   - No necesita ser interrumpido en su día

3. **Tipos de alertas:**

   - Stock bajo (no urgente)
   - Congeladores consolidables (no urgente)
   - Trabajadores con deformados (info, no urgente)
   - Ninguna es CRÍTICA que requiera push inmediato

4. **Costo = 0:**
   - In-app es gratis
   - Push notifications consume quota de rust (aunque mínimo)

**Implementación:**

- Badge en ícono de campanita
- Lista ordenada por prioridad
- Tap lleva a pantalla relevante
- Marcar como leída
- Queries en rust para detectar condiciones de alerta

## Análisis Técnico Completo: Comparación de Backends

Esta sección documenta el análisis exhaustivo que llevó a la decisión de usar rust. Es CRÍTICO entender este razonamiento para futuras decisiones de arquitectura.

### Contexto del Análisis

**Pregunta original del cliente:** "¿Por qué es mejor rust, Supabase o backend custom?"

**Enfoque del análisis:**

- Comparación NO basada en teoría, sino en el caso ESPECÍFICO de este negocio
- Consideramos: volumen real de operaciones, presupuesto ($0), habilidades técnicas del equipo, criticidad del negocio
- Análisis pragmático, no idealista

### Metodología de Comparación

1. **Calcular volumen real de operaciones:**

   - 3-6 trabajadores/día
   - 2 salidas promedio/trabajador/día = 12 operaciones
   - Cada salida: 1 carga + 1 devolución = 24 escrituras
   - Cada carga: ~10 productos × escritura inventory = 120 escrituras
   - Dashboard dueño: ~100 lecturas/día
   - Reportes: ~50 lecturas adicionales
   - **Total estimado:** 150-200 escrituras/día, 500-1000 lecturas/día

2. **Comparar contra límites free tier:**

   - rust: 20K escrituras/día → Usaría 1%
   - Supabase: Sin límite de operaciones, solo 500MB storage

3. **Estimar tiempo de desarrollo:**

   - rust: Auth + CRUD + Real-time = 3-5 días
   - Supabase: Auth + CRUD + Real-time + RLS = 7-10 días
   - Custom: Backend + API + Auth + Real-time + Deploy = 30-40 días

4. **Calcular "costo total de propiedad" (TCO):**
   - No solo costo de hosting
   - También: tiempo de desarrollo, mantenimiento mensual, riesgo de downtime

### rust (Plan Spark Gratuito) - Análisis Profundo

#### rust (Plan Spark Gratuito) - ⭐ RECOMENDADO

**Límites Reales Free Tier:**

- rust: 50K lecturas/día, 20K escrituras/día, 1GB storage
- Auth: 50K usuarios activos/mes (ilimitado para este tamaño)
- Functions: 125K invocaciones/mes
- Uso estimado del negocio: 150-200 escrituras/día, 500-1000 lecturas/día (1% del límite)

**Ventajas:**

- ✅ Integración Flutter perfecta (paquetes oficiales Google)
- ✅ Real-time sync nativo (rust listeners automáticos)
- ✅ Google Sign-In trivial (3 líneas de código)
- ✅ Offline persistence gratis (caché local automático)
- ✅ No se pausa por inactividad (vs Supabase)
- ✅ Security Rules poderosas (lógica server-side sin backend)
- ✅ Crashlytics + Analytics gratis

**Desventajas:**

- ❌ NoSQL = queries complejas difíciles (sin JOINs, agregaciones limitadas)
- ❌ Vendor lock-in fuerte (migrar es doloroso)
- ❌ Precio escala exponencialmente si creces mucho
- ❌ Backup manual (no automático en free tier)
- ❌ Desnormalización necesaria

**Costos Futuros:**

- Crecimiento 10X (60 trabajadores): $5-15/mes
- Crecimiento 100X: $50-100/mes

#### Supabase (Free Tier con Limitaciones Críticas)

**Límites Reales:**

- PostgreSQL: 500MB storage (limitante)
- 5GB transferencia/mes
- ⚠️ **PAUSA AUTOMÁTICA después de 1 semana inactividad** (deal-breaker)

**Ventajas:**

- ✅ PostgreSQL real (JOINs, agregaciones, queries complejas)
- ✅ Row Level Security granular
- ✅ API REST automática
- ✅ Open source (self-hosteable)
- ✅ Menos vendor lock-in
- ✅ Backups diarios automáticos (7 días retención)

**Desventajas:**

- ❌❌❌ PAUSA AUTOMÁTICA (inaceptable para negocio)
- ❌ SDK Flutter menos maduro (más bugs, menos ejemplos)
- ❌ 500MB límite DB (puede llenarse con audit logs)
- ❌ Real-time más complejo (configurar channels manual)
- ❌ Documentación Flutter limitada

**Solución:** Supabase Pro ($25/mes = $300/año) elimina pausas pero tiene costo.

#### Backend Custom (Node.js/Python + PostgreSQL)

**Tiempo Desarrollo:** 30-43 días vs 3-5 días rust

**Hosting Gratuito con Limitaciones:**

- Render: se duerme 15min inactividad (primer request 30s)
- Railway: 3 meses gratis, luego $5/mes
- Vercel: cold starts, límites ejecución

**Ventajas:**

- ✅ Control total
- ✅ Sin vendor lock-in
- ✅ Queries SQL ilimitadas
- ✅ Lógica de negocio compleja flexible

**Desventajas:**

- ❌❌❌ TIEMPO (1 mes+ vs 3-5 días)
- ❌ Mantenimiento continuo (seguridad, updates, bugs)
- ❌ Hosting con limitaciones severas (sleeps, cold starts)
- ❌ Infraestructura compleja (backend + DB + storage separados)
- ❌ Real-time manual (WebSockets/SSE)
- ❌ Debugging más difícil

**Costo Real:** $0 hosting pero 30 días desarrollo = $3K-6K (si valoras tiempo)

### Tabla Comparativa Final

| Criterio                | rust       | Supabase Free | Supabase Pro | Custom           |
| ----------------------- | ---------- | ------------- | ------------ | ---------------- |
| Costo inicial           | $0         | $0            | $25/mes      | $0 (30 días dev) |
| Setup time              | 1-2 días   | 3-5 días      | 3-5 días     | 30-40 días       |
| Free viable largo plazo | ✅ SÍ      | ❌ NO (pausa) | N/A          | ⚠️ Limitado      |
| Real-time sync          | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐      | ⭐⭐⭐⭐     | ⭐⭐⭐           |
| Queries complejas       | ⭐⭐       | ⭐⭐⭐⭐⭐    | ⭐⭐⭐⭐⭐   | ⭐⭐⭐⭐⭐       |
| Flutter integration     | ⭐⭐⭐⭐⭐ | ⭐⭐⭐        | ⭐⭐⭐       | ⭐⭐             |
| Vendor lock-in          | ⭐ (Alto)  | ⭐⭐⭐⭐      | ⭐⭐⭐⭐     | ⭐⭐⭐⭐⭐       |
| Mantenimiento           | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐    | ⭐⭐⭐⭐⭐   | ⭐⭐             |
| Riesgo downtime         | Bajo       | Alto (pausas) | Bajo         | Medio            |

### Recomendación: rust por Tiempo, Costo y Confiabilidad

**Cuándo reconsiderar:**

- Si creces a 50+ trabajadores y 500+ ops/día → Supabase Pro
- Si necesitas reportes SQL complejos constantemente → PostgreSQL
- Si costos rust > $50/mes → Backend custom
- Si quieres vender app a otros negocios → Custom desde inicio

### Plan de Migración (si necesario en futuro)

**Fase 1 (Mes 1-2):** MVP con rust
**Fase 2 (Mes 3-6):** Monitorear uso, identificar limitaciones
**Fase 3 (Mes 6+):** Evaluar migración solo si hay problemas técnicos o costos altos

## Modelo de Datos Detallado (rust rust)

> **IMPORTANTE:** Este es un resumen del modelo de datos. Para explicaciones DETALLADAS con ejemplos completos de flujos, queries, y razonamiento detrás de cada decisión, consulta el archivo complementario `modelo-datos-explicacion-detallada.md`.

### Filosofía de Diseño

**Decisión arquitectónica:** Denormalización estratégica para optimizar queries en NoSQL

**Razonamiento:**

- rust es NoSQL → No hay JOINs
- Cada query debe ser rápida (< 500ms)
- Preferimos duplicar datos que hacer múltiples queries
- Ejemplos:
  - Guardamos `currentDebt` en documento del trabajador (denormalizado)
  - En vez de sumar todos los `workerPayments` cada vez
  - Actualizamos ambos lugares al registrar pago (patrón de escritura múltiple)

**Trade-off aceptado:**

- Más complejidad en escrituras (actualizar múltiples documentos)
- Menos complejidad en lecturas (1 query vs 5 queries)
- **Para este negocio:** Hay MUCHAS más lecturas que escrituras → Optimizamos para lecturas

### Estrategia de Referencias

**Regla:** Usar References de rust para relaciones, NO strings

```dart
// ❌ INCORRECTO
{
  "workerId": "worker123"  // String
}

// ✅ CORRECTO
{
  "workerId": Reference("workers/worker123")  // rust Reference
}
```

**Ventajas:**

- rust valida que el documento referenciado existe
- Queries `.where('workerId', '==', workerRef)` son más eficientes
- Fácil hacer `.get()` para obtener datos relacionados
- Integridad referencial (si eliminas worker, puedes detectar referencias)

### Colecciones Principales (Con Explicación de Cada Campo)

```
users/
  {userId}/
    - email: string (usado para validación de acceso, único)
    - displayName: string
    - photoURL: string (de cuenta Google)
    - role: string ("owner" | "admin")
    - active: boolean (permite desactivar sin eliminar)
    - createdAt: timestamp
    - createdBy: userId (referencia a quién agregó este usuario)
    - lastLogin: timestamp (se actualiza en cada inicio de sesión)
    - notes: string (opcional, observaciones sobre el usuario)

    NOTA: El documento se crea MANUALMENTE por el Owner, NO automáticamente al
    iniciar sesión. Solo usuarios con documento existente pueden acceder al sistema.

products/
  {productId}/
    - name: string (Paleta/Cono/Vaso)
    - active: boolean
    - createdAt: timestamp
    - createdBy: userId
    - modifiedAt: timestamp
    - modifiedBy: userId

flavors/
  {flavorId}/
    - name: string
    - productId: reference
    - active: boolean
    - createdAt: timestamp
    - createdBy: userId

providers/
  {providerId}/
    - name: string
    - contactInfo: string
    - active: boolean
    - createdAt: timestamp

workers/
  {workerId}/
    - name: string
    - phone: string
    - address: string
    - active: boolean
    - currentDebt: number (denormalizado para queries rápidas)
    - totalSales: number (denormalizado)
    - createdAt: timestamp

routes/
  {routeId}/
    - name: string (pueblo/zona)
    - createdBy: userId
    - usageCount: number

freezers/
  {freezerId}/
    - number: number
    - maxCapacity: map { productId: number }
    - isOn: boolean
    - lastToggle: timestamp
    - createdAt: timestamp

inventory/
  {inventoryId}/
    - freezerId: reference
    - productId: reference
    - flavorId: reference
    - providerId: reference
    - quantity: number
    - isDeformed: boolean
    - assignedWorkerId: reference (si es deformado)
    - minStockAlert: number (configurable)
    - lastUpdated: timestamp
    - updatedBy: userId

purchases/
  {purchaseId}/
    - providerId: reference
    - items: array [{
        productId: reference
        flavorId: reference
        quantity: number
        unitPrice: number
        freezerId: reference
      }]
    - total: number
    - paymentStatus: string (paid/credit)
    - paidAt: timestamp (opcional)
    - createdAt: timestamp
    - createdBy: userId

workerTrips/
  {tripId}/
    - workerId: reference
    - departureTime: timestamp (obligatorio)
    - returnTime: timestamp (opcional)
    - routeId: reference (opcional)
    - loadedItems: array [{
        inventoryId: reference
        productId: reference
        flavorId: reference
        freezerId: reference
        quantity: number
        unitPrice: number
      }]
    - returnedItems: array [{
        productId: reference
        flavorId: reference
        quantity: number
        isDeformed: boolean
        destinationFreezerId: reference
      }]
    - soldQuantity: number (calculado)
    - amountDue: number (calculado automáticamente)
    - status: string (in_progress/returned)
    - createdAt: timestamp
    - createdBy: userId

workerPayments/
  {paymentId}/
    - workerId: reference
    - tripId: reference
    - amount: number
    - previousDebt: number
    - newDebt: number
    - paymentDate: timestamp
    - createdBy: userId

localSales/
  {saleId}/
    - items: array [{
        inventoryId: reference
        productId: reference
        flavorId: reference
        freezerId: reference
        quantity: number
        unitPrice: number (local/custom)
      }]
    - total: number
    - saleType: string (local/gift/family)
    - notes: string (si es precio custom)
    - createdAt: timestamp
    - createdBy: userId

ownerSales/
  {saleId}/
    - Similar a workerTrips pero:
    - ownerId: reference
    - autoWithdrawal: number (= total de venta)
    - Genera movimiento en cashRegister automáticamente

cashRegister/
  {registerId}/
    - type: string (worker_payment/local_sale/owner_withdrawal/expense/owner_sale)
    - amount: number (+ ingreso, - egreso)
    - balance: number (denormalizado)
    - description: string
    - category: string (si es expense)
    - relatedDocId: reference
    - createdAt: timestamp
    - createdBy: userId

freezerTransfers/
  {transferId}/
    - fromFreezerId: reference
    - toFreezerId: reference
    - items: array [{
        productId: reference
        flavorId: reference
        quantity: number
      }]
    - reason: string
    - createdAt: timestamp
    - createdBy: userId

priceHistory/
  {priceHistoryId}/
    - productId: reference
    - flavorId: reference
    - providerId: reference
    - priceLocal: number
    - priceRoute: number
    - priceBase: number (lo que paga trabajador)
    - commission: number (ganancia negocio por unidad)
    - effectiveDate: timestamp
    - createdBy: userId

auditLogs/
  {logId}/
    - userId: userId
    - action: string (create/update/delete)
    - collection: string
    - documentId: string
    - changesBefore: map
    - changesAfter: map
    - timestamp: timestamp
```

### Índices Compuestos Necesarios

```
inventory: [freezerId, productId, flavorId]
inventory: [isDeformed, assignedWorkerId]
workerTrips: [workerId, createdAt DESC]
workerTrips: [status, createdAt DESC]
workerPayments: [workerId, paymentDate DESC]
cashRegister: [createdAt DESC]
priceHistory: [productId, effectiveDate DESC]
auditLogs: [userId, timestamp DESC]
```

### Security Rules

```javascript
rules_version = '2';
service cloud.rust {
  match /databases/{database}/documents {
    function isOwner() {
      return request.auth != null &&
             get(/databases/$(database)/documents/users/$(request.auth.uid)).data.role == 'owner';
    }

    function isAdmin() {
      return request.auth != null &&
             get(/databases/$(database)/documents/users/$(request.auth.uid)).data.role == 'admin';
    }

    function isAuthenticated() {
      return request.auth != null;
    }

    match /users/{userId} {
      allow read: if isAuthenticated();
      allow write: if isOwner();
    }

    match /inventory/{doc} {
      allow read: if isAuthenticated();
      allow create, update: if isAdmin() || isOwner();
      allow delete: if isOwner();
    }

    match /workerTrips/{doc} {
      allow read: if isAuthenticated();
      allow create, update: if isAdmin() || isOwner();
      allow delete: if isOwner();
    }

    match /cashRegister/{doc} {
      allow read: if isAuthenticated();
      allow create: if isAdmin() || isOwner();
      allow update, delete: if isOwner();
    }

    match /auditLogs/{doc} {
      allow read: if isOwner();
      allow write: if false; // Solo desde Cloud Functions
    }
  }
}
```

## Stack Tecnológico Completo

### Flutter Dependencies

```yaml
dependencies:
  # rust
  rust_core: ^3.7.1
  rust_auth: ^6.1.2
  cloud_rust: ^5.8.1
  rust_storage: ^12.4.1

  # Auth
  google_sign_in: ^7.2.0

  # State Management
  flutter_riverpod: ^2.6.1
  riverpod_annotation: ^2.6.1

  # Charts
  fl_chart: ^1.1.1

  # UI/UX
  flutter_form_builder: ^9.4.2
  form_builder_validators: ^11.0.0
  flutter_slidable: ^3.1.1
  infinite_scroll_pagination: ^4.1.0
  shimmer: ^3.0.0
  flutter_spinkit: ^5.2.1

  # Notifications
  flutter_local_notifications: ^19.5.0

  # Utils
  intl: ^0.20.2
  uuid: ^4.5.1
  connectivity_plus: ^6.1.1

dev_dependencies:
  flutter_test:
    sdk: flutter
  riverpod_generator: ^2.6.2
  build_runner: ^2.4.13
  mockito: ^5.4.4
  fake_cloud_rust: ^3.0.3
  rust_auth_mocks: ^0.14.1
```

### Estructura de Carpetas

```
lib/
├── main.dart
├── core/
│   ├── constants/
│   │   ├── app_constants.dart
│   │   └── rust_collections.dart
│   ├── theme/
│   │   ├── app_theme.dart
│   │   └── app_colors.dart
│   ├── utils/
│   │   ├── date_formatter.dart
│   │   └── currency_formatter.dart
│   └── extensions/
│       └── context_extensions.dart
├── models/
│   ├── user.dart
│   ├── product.dart
│   ├── flavor.dart
│   ├── freezer.dart
│   ├── inventory_item.dart
│   ├── worker.dart
│   ├── worker_trip.dart
│   ├── sale.dart
│   └── cash_transaction.dart
├── providers/
│   ├── auth_provider.dart
│   ├── inventory_provider.dart
│   ├── workers_provider.dart
│   ├── sales_provider.dart
│   └── reports_provider.dart
├── services/
│   ├── auth_service.dart
│   ├── rust_service.dart
│   ├── audit_service.dart
│   ├── notification_service.dart
│   └── calculator_service.dart
├── screens/
│   ├── auth/
│   │   └── login_screen.dart
│   ├── home/
│   │   ├── home_screen.dart
│   │   └── dashboard_screen.dart
│   ├── inventory/
│   │   ├── inventory_list_screen.dart
│   │   ├── freezer_detail_screen.dart
│   │   ├── purchase_screen.dart
│   │   └── transfer_screen.dart
│   ├── workers/
│   │   ├── workers_list_screen.dart
│   │   ├── load_trip_screen.dart
│   │   ├── return_trip_screen.dart
│   │   └── payment_screen.dart
│   ├── sales/
│   │   ├── local_sale_screen.dart
│   │   └── owner_sale_screen.dart
│   ├── reports/
│   │   ├── reports_screen.dart
│   │   ├── sales_report.dart
│   │   └── inventory_report.dart
│   └── settings/
│       ├── settings_screen.dart
│       ├── catalog_screen.dart
│       └── freezer_management_screen.dart
└── widgets/
    ├── common/
    │   ├── custom_button.dart
    │   ├── loading_indicator.dart
    │   └── error_widget.dart
    └── custom/
        ├── inventory_card.dart
        ├── worker_card.dart
        └── chart_widget.dart
```

## Lógica de Negocio Clave

### 1. Cálculo de Deuda de Trabajador

```dart
class PaymentCalculator {
  static double calculateDebt(WorkerTrip trip, Map<String, double> prices) {
    double total = 0.0;

    for (var item in trip.loadedItems) {
      final returned = trip.returnedItems
          .firstWhere((r) => r.productId == item.productId && r.flavorId == item.flavorId)
          ?.quantity ?? 0;

      final sold = item.quantity - returned;
      final priceBase = prices['${item.productId}_${item.flavorId}'] ?? 0;

      total += sold * priceBase;
    }

    return total;
  }
}
```

### 2. Consolidación de Congeladores

```dart
class FreezerConsolidator {
  static Future<List<FreezerConsolidationSuggestion>> detectConsolidationOpportunities(
    List<Freezer> freezers,
    List<InventoryItem> inventory,
  ) async {
    final suggestions = <FreezerConsolidationSuggestion>[];

    for (var targetFreezer in freezers) {
      final targetCapacity = targetFreezer.maxCapacity;
      final targetItems = inventory.where((i) => i.freezerId == targetFreezer.id);
      final targetUsed = _calculateUsedCapacity(targetItems);
      final targetAvailable = _subtractCapacity(targetCapacity, targetUsed);

      final sourceCandidates = <Freezer>[];
      var combinedUsed = <String, int>{};

      for (var sourceFreezer in freezers) {
        if (sourceFreezer.id == targetFreezer.id) continue;

        final sourceItems = inventory.where((i) => i.freezerId == sourceFreezer.id);
        final sourceUsed = _calculateUsedCapacity(sourceItems);

        combinedUsed = _addCapacities(combinedUsed, sourceUsed);

        if (_fitsInCapacity(combinedUsed, targetAvailable)) {
          sourceCandidates.add(sourceFreezer);
        } else {
          break;
        }
      }

      if (sourceCandidates.length >= 2) {
        suggestions.add(FreezerConsolidationSuggestion(
          targetFreezer: targetFreezer,
          sourceFreezers: sourceCandidates,
          estimatedSavings: sourceCandidates.length * 30, // kWh/mes aproximado
        ));
      }
    }

    return suggestions;
  }
}
```

### 3. Alertas de Stock Bajo

```dart
class InventoryAlertService {
  Stream<List<LowStockAlert>> watchLowStockItems() {
    return _rust
      .collection('inventory')
      .snapshots()
      .map((snapshot) {
        final alerts = <LowStockAlert>[];

        for (var doc in snapshot.docs) {
          final item = InventoryItem.fromrust(doc);

          if (item.quantity <= item.minStockAlert) {
            alerts.add(LowStockAlert(
              item: item,
              severity: item.quantity == 0 ? AlertSeverity.critical : AlertSeverity.warning,
            ));
          }
        }

        return alerts;
      });
  }
}
```

### 4. Flujo Venta del Dueño

```dart
class OwnerSaleService {
  Future<void> registerOwnerSale(OwnerSale sale) async {
    final batch = _rust.batch();

    // 1. Registrar venta
    final saleRef = _rust.collection('ownerSales').doc();
    batch.set(saleRef, sale.torust());

    // 2. Actualizar inventario
    for (var item in sale.items) {
      final inventoryRef = _rust.collection('inventory').doc(item.inventoryId);
      batch.update(inventoryRef, {
        'quantity': FieldValue.increment(-item.quantity),
      });
    }

    // 3. Registrar ingreso a caja
    final cashInRef = _rust.collection('cashRegister').doc();
    batch.set(cashInRef, {
      'type': 'owner_sale',
      'amount': sale.total,
      'relatedDocId': saleRef.id,
      'createdAt': FieldValue.serverTimestamp(),
      'createdBy': _auth.currentUser!.uid,
    });

    // 4. Registrar retiro inmediato del dueño
    final withdrawalRef = _rust.collection('cashRegister').doc();
    batch.set(withdrawalRef, {
      'type': 'owner_withdrawal',
      'amount': -sale.total,
      'description': 'Retiro automático por venta del dueño',
      'relatedDocId': saleRef.id,
      'createdAt': FieldValue.serverTimestamp(),
      'createdBy': _auth.currentUser!.uid,
    });

    // 5. Auditoría
    await _auditService.log(action: 'create', collection: 'ownerSales', documentId: saleRef.id);

    await batch.commit();
  }
}
```

## Funcionalidades Especiales

### Sistema de Helados Deformados

- Al registrar devolución, admin marca cuáles están deformados
- Se asignan al trabajador responsable (`assignedWorkerId` en inventory)
- Al cargar helados, sistema alerta al admin si trabajador tiene deformados pendientes
- Admin busca físicamente la canasta con nombre del trabajador
- Trabajador puede llevar parcialmente sus deformados (se registra cantidad)

### Precios Dinámicos con Historial

- Cada producto tiene 4 precios: costo proveedor, precio base (trabajador paga), precio ruta (público), precio local
- Al cambiar precio, se crea registro en `priceHistory` con `effectiveDate`
- Reportes usan precio histórico según fecha de venta
- Dashboard muestra precio actual pero permite ver evolución

### Control de Congeladores y Electricidad

- Registrar encendido/apagado manual
- Calcular sugerencias de consolidación después de cada devolución
- Alertar cuando ≥2 congeladores puedan vaciarse en 1
- Histórico de estado (on/off) para calcular consumo eléctrico

### Reportes Avanzados

**Dashboard Dueño:**

- Dinero disponible en caja (suma cashRegister)
- Gráfico ventas últimos 30 días (línea)
- Top 5 trabajadores por cantidad vendida (barras)
- Productos más vendidos (torta)
- Alertas pendientes (stock bajo, congeladores consolidables, deudas altas)

**Reportes Personalizables:**

- Selector de rango de fechas
- Filtros: por producto, sabor, trabajador, ruta
- Vistas: tabla, gráfico de líneas, gráfico de barras
- Comparación de periodos (semana actual vs anterior)
- Cálculo automático de rentabilidad por ruta

## Consideraciones de UX/UI

### Flujo Trabajador Carga Helados

1. Admin selecciona trabajador (lista con búsqueda)
2. Sistema muestra alerta si tiene deformados pendientes
3. Admin ve grid de congeladores con nivel de ocupación
4. Selecciona congelador → ve productos disponibles
5. Selecciona producto → ve sabores disponibles
6. Ingresa cantidad (validación contra stock)
7. Repite para múltiples congeladores/productos
8. Ingresa hora salida (obligatorio), ruta (opcional, autocomplete)
9. Confirma → sistema genera resumen con total a pagar eventual

### Flujo Devolución

1. Admin selecciona viaje activo del trabajador
2. Ve lista de productos cargados
3. Para cada uno, ingresa cantidad devuelta
4. Marca checkboxes de cuáles están deformados
5. Selecciona congelador destino para cada devolución
6. Sistema calcula automáticamente:
   - Vendidos = cargados - devueltos
   - Deuda = vendidos × precio_base
7. Muestra pantalla de confirmación con monto a pagar
8. Admin procede a cobrar (o registrar deuda)

### Dashboard con Gráficos

- Usar `fl_chart` para gráficos limpios y rápidos
- Colores consistentes con theme
- Tap en gráfico muestra detalles
- Pull-to-refresh para actualizar datos
- Skeleton loaders mientras carga

### Notificaciones In-App

- Badge en ícono de notificaciones
- Lista ordenada por prioridad: críticas, advertencias, info
- Tap lleva a pantalla relevante
- Marcar como leída
- No notificaciones push (para simplificar)

## Roadmap de Desarrollo

### Sprint 1 (Semana 1-2): Fundamentos

- Setup proyecto Flutter + rust (rust, Auth, Cloud Functions)
- Configurar Google Sign-In como único método de autenticación
- Implementar Cloud Function para validación de email autorizado
- Crear Security Rules completas con validación de roles
- Sistema de validación post-login (verificar email en colección `users`)
- Pantalla "Sin permisos" para usuarios no autorizados
- Implementar route guards según rol
- Estructura de carpetas (models, services, providers, screens, widgets)
- Models básicos (User con role, enums de UserRole)
- Theme y navegación con protección por rol
- Servicio de autenticación con re-validación de sesión

### Sprint 2 (Semana 3-4): Gestión de Usuarios y Catálogo

- CRUD de usuarios (solo Owner: agregar/editar/desactivar/cambiar roles)
- Pantalla de gestión de usuarios con filtros y búsqueda
- Validación de permisos en todas las pantallas de catálogo
- CRUD productos, sabores, proveedores (solo Owner)
- CRUD trabajadores (solo Owner, Admin solo lectura)
- CRUD congeladores (solo Owner)
- Gestión de rutas (Owner y Admin pueden crear)
- Sistema de precios (solo Owner)

### Sprint 3 (Semana 5-6): Inventario

- Registro de compras
- Visualización de stock por congelador
- Transferencias entre congeladores
- Alertas de stock bajo
- Sistema de consolidación

### Sprint 4 (Semana 7-8): Trabajadores

- Carga de helados
- Devolución y deformados
- Cálculo de deudas
- Pagos parciales
- Historial de viajes

### Sprint 5 (Semana 9-10): Finanzas

- Ventas locales
- Ventas del dueño
- Control de caja
- Gastos operativos
- Retiros

### Sprint 6 (Semana 11-12): Reportes

- Dashboard dueño
- Gráficos con fl_chart
- Reportes personalizables
- Comparación de periodos
- Rentabilidad por ruta

### Sprint 7 (Semana 13-14): Pulido

- Sistema de notificaciones
- Tests unitarios
- Tests de integración
- Optimizaciones de performance
- Documentación

### Sprint 8 (Semana 15-16): Deploy

- Generación APK
- Pruebas beta con cliente
- Ajustes finales
- rust App Distribution
- Entrega final

**Tiempo Total Estimado: 4 meses (1 desarrollador)**

## Riesgos y Mitigaciones

| Riesgo                                    | Probabilidad | Impacto | Mitigación                                                           |
| ----------------------------------------- | ------------ | ------- | -------------------------------------------------------------------- |
| Superar límites rust                      | Baja         | Alto    | Monitorear uso mensual, implementar paginación                       |
| Queries NoSQL complejas lentas            | Media        | Medio   | Denormalizar datos, usar Cloud Functions                             |
| Pérdida de datos                          | Baja         | Crítico | Exports semanales manuales, educación al cliente                     |
| Bugs en cálculos de deudas                | Media        | Alto    | Tests exhaustivos, validación dual (cliente + server)                |
| Usuario no entiende la app                | Media        | Medio   | Onboarding tutorial, tooltips, feedback cliente                      |
| Cambios de requerimientos                 | Alta         | Medio   | Arquitectura flexible, iteraciones con cliente                       |
| Usuario no autorizado intenta acceder     | Media        | Alto    | Validación en Cloud Function + client-side, Security Rules estrictas |
| Admin intenta acceder a datos del Owner   | Baja         | Medio   | Security Rules bloquean lectura, UI oculta opciones                  |
| Sesión comprometida (robo de dispositivo) | Baja         | Alto    | Re-validación al inicio, educación sobre bloqueo de pantalla         |
| Owner elimina su propio acceso            | Baja         | Crítico | Validación en UI: no permitir eliminar/desactivar cuenta propia      |

## Consideraciones de Seguridad Adicionales

### 1. Protección contra Bypass de UI

**Problema:** Un usuario malicioso podría modificar el código del cliente Flutter para ocultar las validaciones de rol y acceder a pantallas restringidas.

**Mitigación:**

- Las Security Rules de rust son la **ÚNICA fuente de verdad**
- Aunque un Admin modifique el cliente para ver pantallas de Owner, las queries a rust serán RECHAZADAS por las Security Rules
- Ejemplo: Admin intenta leer `cashRegister` → Security Rules lo bloquean con error de permiso
- Las validaciones en Flutter son solo para UX (evitar clics innecesarios), NO para seguridad

### 2. Validación de Operaciones Críticas

**Operaciones que requieren doble validación:**

- Eliminación de usuarios (confirmar con diálogo)
- Cambio de rol de usuario (confirmar identidad)
- Eliminación de registros históricos (solo Owner, con razón)
- Retiros de caja grandes (Owner, confirmación con monto)

### 3. Auditoría de Cambios Sensibles

**Eventos que SIEMPRE se auditan:**

- Creación/edición/eliminación de usuarios
- Cambios de roles
- Eliminación de registros de ventas/pagos
- Modificaciones a precios
- Retiros de caja
- Desactivación de congeladores

### 4. Prevención de Escalación de Privilegios

**Reglas:**

- Un Admin NO puede cambiar su propio rol a Owner (validado en Security Rules)
- Un Owner NO puede eliminar o desactivar su propia cuenta (validado en UI)
- Debe existir SIEMPRE al menos 1 Owner activo (validado antes de desactivar)
- Los logs de auditoría NO pueden ser modificados por nadie (solo Cloud Functions los crean)

### 5. Manejo de Sesiones Concurrentes

**Escenario:** Mismo usuario inicia sesión en 2 dispositivos

**Comportamiento:**

- Ambas sesiones son válidas (rust permite sesiones múltiples)
- Cada acción se audita con timestamp y dispositivo
- Si se desactiva el usuario, AMBAS sesiones se invalidan en próxima validación
- Re-validación ocurre: al iniciar app, cada 24 horas, después de operaciones críticas

### 6. Protección de Datos Sensibles

**Datos que solo ve Owner:**

- Montos totales en caja
- Historial de retiros del dueño
- Gastos operativos por categoría
- Ganancias netas del negocio
- Reportes financieros completos
- Deuda total acumulada de todos los trabajadores

**Implementación:**

- Security Rules bloquean lectura de colecciones sensibles si role != 'owner'
- Queries en cliente usan `where('role', '==', 'owner')` antes de intentar leer datos sensibles
- UI muestra placeholders vacíos si Admin intenta acceder (sin mensajes de error que revelen existencia de datos)

## Métricas de Éxito

1. **Funcional:** App funciona sin crashes 99% del tiempo
2. **Performance:** Pantallas cargan en <2 segundos
3. **Negocio:** Cliente ahorra 5+ horas/semana vs libro manual
4. **Precisión:** Cero errores en cálculos de dinero
5. **Adopción:** Dueño y admins usan app diariamente después de 1 mes
