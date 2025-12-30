# Lenovo IdeaPad - Guía de Temperaturas

## Temperaturas Máximas de Seguridad (CPU)

| Estado | Temperatura | Evaluación |
|--------|-------------|------------|
| Reposo (idle) | 30-50°C | Normal |
| Bajo carga | 70-85°C | Aceptable |
| Zona de alerta | 85-90°C | Mejorar refrigeración |
| Crítico | >90°C | Riesgo de throttling y daño |

> El portátil se apaga automáticamente en temperaturas críticas para proteger los componentes.

## Temperaturas Recomendadas para Comodidad

### Superficie del Teclado

| Temperatura | Sensación |
|-------------|-----------|
| < 35°C | Confortable |
| 35-40°C | Tibio pero cómodo |
| 40-45°C | Caliente, manos sudorosas |
| > 45°C | Muy caliente, uso prolongado difícil |

**Recomendación:** Mantener bajo **40°C** para uso prolongado sin incomodidad.

## Temperaturas Óptimas para Máxima Vida Útil

| Rango | Temperatura CPU | Impacto en longevidad |
|-------|-----------------|----------------------|
| **Óptimo** | 40-60°C | Máxima vida útil |
| Aceptable | 60-70°C | Vida útil normal |
| Tolerable | 70-80°C | Degradación gradual acelerada |
| Perjudicial | > 80°C constante | Reduce significativamente la vida útil |

### Por qué importa mantener temperaturas bajas

- Por cada 10°C de aumento, las reacciones de degradación se duplican
- El calor acelera la electromigración (erosión de circuitos)
- La pasta térmica se degrada más rápido
- Los SSDs también sufren con temperaturas elevadas constantes

## Objetivos por Tipo de Uso

| Uso | CPU objetivo | Superficie objetivo |
|-----|--------------|---------------------|
| Reposo/oficina | < 50°C | < 35°C |
| Trabajo moderado | < 65°C | < 40°C |
| Gaming/renderizado | < 75°C | < 45°C |

## Acciones si las Temperaturas son Altas

1. **Base refrigerante** - Puede reducir 10-15°C
2. **Limpiar ventiladores y rejillas** - El polvo reduce eficiencia
3. **Renovar pasta térmica** - Si tiene más de 2-3 años
4. **Elevar el portátil** - Superficie dura y plana
5. **Evitar superficies blandas** - Cama, sofá bloquean ventilación

## Monitoreo de Temperaturas

```bash
# Instalar sensores (si no está instalado)
sudo apt install lm-sensors

# Detectar sensores
sudo sensors-detect

# Ver temperaturas actuales
sensors

# Monitoreo continuo cada 2 segundos
watch -n 2 sensors
```

## Referencias

- [Lenovo CPU Temperature Guide](https://www.lenovo.com/us/en/glossary/what-is-cpu-temperature/)
- [Ideal Temperature For Laptop CPU](https://ms.codes/blogs/computer-hardware/ideal-temperature-for-laptop-cpu)
- [How Temperature Affects Computer Lifespan](https://scot-comp.co.uk/how-temperature-affects-computer-performance-and-lifespan/)
