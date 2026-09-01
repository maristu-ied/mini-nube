# Esquema Base de Datos - Mini Nube

## Jerarquía de dispositivos

```
NCU (Network Control Unit)
├── HSU (sensores meteorológicos)
└── TCU (Tracker Control Unit)
```

Una BD SQLite única puede contener datos de múltiples NCUs. El `ncu_id` se pasa como parámetro en la ingesta.

## Tablas

### `dispositivos` (registro de dispositivos)

| Columna     | Tipo    | Descripción                    |
|-------------|---------|--------------------------------|
| id          | INTEGER | PK autoincremental             |
| ncu_id      | TEXT    | Identificador de la NCU        |
| tipo        | TEXT    | `'NCU'`, `'HSU'` o `'TCU'`    |
| device_id   | TEXT    | Identificador del dispositivo  |
| descripcion | TEXT    | Opcional                       |
| created_at  | INTEGER | Unix epoch (segundos) UTC      |

Restricción: `UNIQUE(ncu_id, tipo, device_id)`

### `datos_ncu` (estado general NCU)

| Columna           | Tipo    | Descripción              |
|-------------------|---------|--------------------------|
| dispositivo_id    | INTEGER | FK a dispositivos        |
| timestamp         | INTEGER | Unix epoch (segundos) UTC|
| mqtt_online       | INTEGER | 0/1                      |
| gw1_online        | INTEGER | 0/1                      |
| gw2_online        | INTEGER | 0/1                      |
| ups_power_ok      | INTEGER | 0/1                      |
| ups_battery_low   | INTEGER | 0/1                      |
| stop_button       | INTEGER | 0/1                      |
| bluetooth_enabled | INTEGER | 0/1                      |

### `datos_ncu_sensor` y `datos_hsu` (mismo esquema)

| Columna              | Tipo    | Descripción                   |
|----------------------|---------|-------------------------------|
| dispositivo_id       | INTEGER | FK a dispositivos             |
| timestamp            | INTEGER | Unix epoch (segundos) UTC     |
| main_battery         | INTEGER | Tensión batería (mV)          |
| internal_temp        | REAL    | Temperatura interna (°C)      |
| wind_speed           | REAL    | Velocidad viento (m/s)        |
| wind_direction       | REAL    | Dirección viento (°)          |
| wind_level           | INTEGER | Nivel de viento               |
| snow_level           | REAL    | Nivel de nieve                |
| irradiance           | INTEGER | Irradiancia (W/m²)            |
| wind_alarm           | INTEGER | 0/1                           |
| gusty_wind_alarm     | INTEGER | 0/1                           |
| snow_alarm           | INTEGER | 0/1                           |
| snow_sensor_com_error| INTEGER | 0/1                           |

### `datos_tcu` (telemetría tracker)

| Columna                        | Tipo    | Descripción                    |
|--------------------------------|---------|--------------------------------|
| dispositivo_id                 | INTEGER | FK a dispositivos              |
| timestamp                      | INTEGER | Unix epoch (segundos) UTC      |
| main_state                     | TEXT    | Estado principal (AUTO, etc.)  |
| backtracking                   | INTEGER | 0/1                            |
| wind_from_east                 | INTEGER | 0/1                            |
| active_security_position       | INTEGER | Posición de seguridad activa   |
| angle                          | REAL    | Ángulo actual (°)              |
| target_angle                   | REAL    | Ángulo objetivo (°)            |
| soc                            | INTEGER | Estado de carga (%)            |
| remaining_capacity             | INTEGER | Capacidad restante (mAh)       |
| ps_voltage                     | INTEGER | Tensión power section (mV)     |
| ps_current                     | INTEGER | Corriente power section (mA)   |
| voltage                        | INTEGER | Tensión (mV)                   |
| current                        | INTEGER | Corriente (mA)                 |
| motor_voltage                  | INTEGER | Tensión motor (mV)             |
| motor_current                  | INTEGER | Corriente motor (mA)           |
| motor_current_peak             | INTEGER | Pico corriente motor (mA)      |
| motor_state                    | TEXT    | Estado motor (OFF, CW, CCW)    |
| motor_pwm                      | REAL    | PWM motor (%)                  |
| daily_motor_power_consumption  | INTEGER | Consumo diario motor (Wh)      |
| pcb_temp                       | REAL    | Temperatura PCB (°C)           |
| battery_temp                   | REAL    | Temperatura batería (°C)       |
| alarms_1                       | INTEGER | Bitmap alarmas grupo 1         |
| alarms_2                       | INTEGER | Bitmap alarmas grupo 2         |
| hw_alarms                      | INTEGER | Bitmap alarmas hardware        |
| system_monitor_status          | INTEGER | Estado monitor sistema         |
| system_monitor_flags           | INTEGER | Flags monitor sistema          |
| power_section_alarms           | INTEGER | Bitmap alarmas power section   |

### `ncu_event_log` (eventos)

| Columna        | Tipo    | Descripción                |
|----------------|---------|----------------------------|
| dispositivo_id | INTEGER | FK a dispositivos          |
| timestamp      | INTEGER | Unix epoch (segundos) UTC  |
| evento         | TEXT    | Texto libre del evento     |

Nota: el CSV de origen no tiene cabecera, formato `timestamp;evento`.

### `ingesta_log` (auditoría de ingesta)

| Columna          | Tipo    | Descripción                    |
|------------------|---------|--------------------------------|
| ncu_id           | TEXT    | NCU de origen                  |
| fichero          | TEXT    | Nombre del fichero CSV         |
| tipo_datos       | TEXT    | Tabla destino                  |
| filas_insertadas | INTEGER | Total de filas procesadas (nuevas + actualizadas) |
| filas_nuevas      | INTEGER | Filas insertadas por primera vez |
| filas_actualizadas | INTEGER | Filas ya existentes reemplazadas |
| timestamp_inicio | INTEGER | Primer timestamp del fichero (epoch) |
| timestamp_fin    | INTEGER | Último timestamp del fichero (epoch) |
| ingested_at      | INTEGER | Momento de la ingesta (epoch UTC)    |

## Índices

Todas las tablas de datos tienen un índice único `(dispositivo_id, timestamp)` para consultas eficientes por dispositivo y rango temporal, y para evitar duplicados en la ingesta. `ncu_event_log` usa `(dispositivo_id, timestamp, evento)` ya que puede haber varios eventos en el mismo timestamp.

## Decisiones de diseño

- **Timestamps**: almacenados en UTC como INTEGER (Unix epoch, segundos), convertidos desde el formato ISO-8601 de los CSV en la ingesta
- **WAL mode**: activado para mejor rendimiento en lecturas concurrentes
- **Foreign keys**: activadas para integridad referencial
- **Booleanos**: almacenados como INTEGER (0/1)
- **Un solo fichero .db**: sin particionado mensual por ahora
- **`ncu_id` como parámetro**: se pasa al ingestar, no viene en los CSV
- **NCU_SENSORS y HSU**: mismas columnas, tablas separadas por claridad semántica
