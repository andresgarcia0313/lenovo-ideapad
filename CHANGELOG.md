# Changelog - Lenovo IdeaPad

## [2025-12-30] v2.0 - Optimización para Confort de Teclado

### Objetivo
Teclado a temperatura de dedos (~35°C) manteniendo máximo rendimiento posible.

### Análisis Físico Realizado
- Modelo de transferencia de calor: `T_teclado = T_amb + (T_CPU - T_amb) × 0.45`
- Temperatura dedos humanos: 30-33°C
- Para teclado ~35°C se requiere CPU < 45°C

### Nuevos Modos

| Modo | Comando | Teclado Est. | Rendimiento | Uso |
|------|---------|--------------|-------------|-----|
| **COMFORT** | `sudo cpu-mode comfort` | ~35°C | 60% (2.6GHz) | Uso diario |
| PERFORMANCE | `sudo cpu-mode performance` | ~45°C+ | 100% (4.4GHz) | Videollamadas |
| BALANCED | `sudo cpu-mode balanced` | ~40°C | 75% (3.3GHz) | Balance |
| QUIET | `sudo cpu-mode quiet` | ~33°C | 40% (1.8GHz) | Silencio |
| AUTO | `sudo cpu-mode auto` | Variable | Variable | Automático |

### Nuevos Umbrales en Modo AUTO

| CPU | Teclado Est. | Rendimiento | Modo |
|-----|--------------|-------------|------|
| < 40°C | ~33°C | 85% | COOL |
| 40-45°C | ~35°C | 70% | COMFORT |
| 45-50°C | ~37°C | 60% | OPTIMAL |
| 50-55°C | ~39°C | 50% | WARM |
| 55-60°C | ~41°C | 40% | HOT |
| 60-70°C | ~44°C | 35% | COOLING |
| > 70°C | ~48°C | 25% | CRITICAL |

### Cambios Técnicos
- Añadida estimación de temperatura de teclado en logs y status
- Nuevos umbrales más agresivos para mantener confort
- Modo COMFORT añadido como opción manual
- Log ahora muestra: `CPU:XXºC → Teclado:~XXºC`

---

## [2025-12-30] v1.0 - Implementación de Gestión Térmica Automática

### Contexto
El equipo Lenovo IdeaPad con Intel i5-1235U presentaba problemas de calentamiento excesivo, afectando la comodidad de uso y potencialmente la vida útil del equipo.

### Cambios Realizados

#### Scripts Creados
- `scripts/thermal-management/thermal-manager.sh` - Script principal de gestión térmica
- `scripts/thermal-management/cpu-mode` - Comando para cambio manual de modos
- `scripts/thermal-management/install-thermal-manager.sh` - Instalador automático

#### Configuraciones
- `docs/configuraciones/thermal-conf.xml` - Configuración de thermald con trip points optimizados
- `docs/configuraciones/systemd-services/thermal-manager.service` - Servicio systemd
- `docs/configuraciones/systemd-services/thermal-manager.timer` - Timer cada 30 segundos

#### Documentación
- `docs/THERMAL-MANAGEMENT.md` - Documentación completa del sistema

### Servicios Modificados

| Servicio | Acción | Motivo |
|----------|--------|--------|
| thermal-manager.timer | Habilitado | Gestión térmica cada 30s |
| thermald | Reconfigurado | Trip points optimizados |
| powertop-autotune | Deshabilitado | Conflicto con gestión térmica |

### Archivos Instalados en el Sistema

```
/usr/local/bin/thermal-manager.sh
/usr/local/bin/cpu-mode
/etc/systemd/system/thermal-manager.service
/etc/systemd/system/thermal-manager.timer
/etc/thermald/thermal-conf.xml
```

### Comportamiento Implementado

| Temperatura | Rendimiento | Modo |
|-------------|-------------|------|
| < 45°C | 100% | BOOST |
| 45-55°C | 90% | OPTIMAL |
| 55-65°C | 75% | BALANCED |
| 65-75°C | 60% | WARM |
| 75-82°C | 45% | HOT |
| > 82°C | 30% | CRITICAL |

### Consideraciones Técnicas

1. **intel_pstate en modo active**: A diferencia de Lenovo G400 (passive), este equipo usa intel_pstate active, requiriendo control via `max_perf_pct` en lugar de `scaling_max_freq`

2. **Sin TLP**: Se decidió no usar TLP porque:
   - Conflicto con intel_pstate active
   - power-profiles-daemon ya está presente
   - Control más directo con scripts propios

3. **Basado en experiencia de Lenovo G400**: Se adaptó el sistema probado en `/home/andres/Desarrollo/k8s/lenovo-g400/scripts/thermal-optimization/`

### Comandos de Uso

```bash
# Ver estado
cpu-mode status

# Videollamadas (máximo rendimiento)
sudo cpu-mode performance

# Volver a automático
sudo cpu-mode auto

# Modo silencioso
sudo cpu-mode quiet

# Ver logs
tail -f /var/log/thermal-manager.log
```

### Verificación Post-Instalación

```bash
# Verificar timer activo
systemctl status thermal-manager.timer

# Verificar última ejecución
tail -5 /var/log/thermal-manager.log

# Ver modo actual
cpu-mode status
```

### Rollback (si es necesario)

```bash
sudo systemctl stop thermal-manager.timer
sudo systemctl disable thermal-manager.timer
sudo rm /usr/local/bin/thermal-manager.sh
sudo rm /usr/local/bin/cpu-mode
sudo rm /etc/systemd/system/thermal-manager.*
sudo rm /etc/thermald/thermal-conf.xml
sudo systemctl daemon-reload
sudo systemctl restart thermald
sudo systemctl enable powertop-autotune.service
```
